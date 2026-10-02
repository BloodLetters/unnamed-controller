use super::manager::DebuggerCore;
#[cfg(not(target_arch = "wasm32"))]
use super::rsp::*;
use super::types::CpuRegisters;
#[cfg(not(target_arch = "wasm32"))]
use super::types::WatchpointKind;

#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::net::{TcpListener, TcpStream};

/// Built-in GDB Remote Serial Protocol (RSP) TCP server.
pub struct GdbServer {
    pub port: u16,
    pub is_running: bool,
    #[cfg(not(target_arch = "wasm32"))]
    listener: Option<TcpListener>,
    #[cfg(not(target_arch = "wasm32"))]
    client: Option<TcpStream>,
    rx_buffer: Vec<u8>,
}

impl Default for GdbServer {
    fn default() -> Self {
        Self::new(3333)
    }
}

impl GdbServer {
    /// Constructs a new GDB RSP server bound to the given local port.
    pub fn new(port: u16) -> Self {
        Self {
            port,
            is_running: false,
            #[cfg(not(target_arch = "wasm32"))]
            listener: None,
            #[cfg(not(target_arch = "wasm32"))]
            client: None,
            rx_buffer: Vec::new(),
        }
    }

    /// Starts listening for incoming GDB remote client connections.
    pub fn start(&mut self) -> Result<(), String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let addr = format!("127.0.0.1:{}", self.port);
            let listener = TcpListener::bind(&addr).map_err(|e| e.to_string())?;
            listener.set_nonblocking(true).map_err(|e| e.to_string())?;
            self.listener = Some(listener);
            self.is_running = true;
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.is_running = false;
            Err("TCP GDB server unsupported on WASM target".into())
        }
    }

    /// Stops the server and disconnects active clients.
    pub fn stop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.client = None;
            self.listener = None;
        }
        self.is_running = false;
        self.rx_buffer.clear();
    }

    /// Non-blocking polling step to process incoming GDB client packets and dispatch debugger state.
    pub fn poll_and_process(
        &mut self,
        debugger: &mut DebuggerCore,
        regs: &CpuRegisters,
        memory: &mut [u8],
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if !self.is_running {
                return;
            }

            if self.client.is_none()
                && let Some(listener) = &self.listener
                && let Ok((stream, _)) = listener.accept()
            {
                let _ = stream.set_nonblocking(true);
                self.client = Some(stream);
            }

            if let Some(stream) = &mut self.client {
                let mut buf = [0u8; 1024];
                match stream.read(&mut buf) {
                    Ok(0) => {
                        self.client = None;
                    }
                    Ok(n) => {
                        self.rx_buffer.extend_from_slice(&buf[..n]);
                        self.process_buffer(debugger, regs, memory);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(_) => {
                        self.client = None;
                    }
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = (debugger, regs, memory);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Processes queued incoming data in the buffer and responds to GDB RSP commands.
    fn process_buffer(
        &mut self,
        debugger: &mut DebuggerCore,
        regs: &CpuRegisters,
        memory: &mut [u8],
    ) {
        let mut drained = 0usize;
        let mut payload = None;

        if let Some(start) = self.rx_buffer.iter().position(|&byte| byte == b'$') {
            if let Some(hash_offset) = self.rx_buffer[start..]
                .iter()
                .position(|&byte| byte == b'#')
            {
                let packet_end = start + hash_offset + 3;
                if packet_end <= self.rx_buffer.len() {
                    if let Ok(raw_packet) = std::str::from_utf8(&self.rx_buffer[start..packet_end])
                    {
                        payload = decode_packet(raw_packet).map(str::to_string);
                    }
                    drained = packet_end;
                }
            }
        } else if let Some(plus_index) = self.rx_buffer.iter().position(|&byte| byte == b'+') {
            drained = plus_index + 1;
        }

        if drained == 0 {
            return;
        }

        self.rx_buffer.drain(..drained);
        if let Some(payload) = payload {
            self.send_ack();
            let response = self.handle_command(&payload, debugger, regs, memory);
            self.send_packet(&response);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Sends a GDB protocol positive acknowledgment (+).
    fn send_ack(&mut self) {
        if let Some(stream) = &mut self.client {
            let _ = stream.write_all(b"+");
            let _ = stream.flush();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Encodes and transmits a framed RSP packet to the active GDB client.
    fn send_packet(&mut self, payload: &str) {
        if let Some(stream) = &mut self.client {
            let packet = encode_packet(payload);
            let _ = stream.write_all(packet.as_bytes());
            let _ = stream.flush();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Evaluates an RSP command and formats the matching GDB response.
    fn handle_command(
        &self,
        payload: &str,
        debugger: &mut DebuggerCore,
        regs: &CpuRegisters,
        memory: &mut [u8],
    ) -> String {
        match parse_command(payload) {
            RspCommand::QuerySupported => "PacketSize=1000;hwbreak+;qXfer:memory-map:read-".into(),
            RspCommand::HaltReason => "S05".into(),
            RspCommand::ReadRegisters => format_registers(regs),
            RspCommand::ReadMemory { addr, len } => {
                let start = (addr as usize).min(memory.len());
                let end = (start + len).min(memory.len());
                format_memory_hex(&memory[start..end])
            }
            RspCommand::WriteMemory { addr, data } => {
                let start = addr as usize;
                for (i, &b) in data.iter().enumerate() {
                    if start + i < memory.len() {
                        memory[start + i] = b;
                    }
                }
                "OK".into()
            }
            RspCommand::InsertBreakpoint { addr } => {
                debugger.add_breakpoint(addr);
                "OK".into()
            }
            RspCommand::RemoveBreakpoint { addr } => {
                debugger.remove_breakpoint(addr);
                "OK".into()
            }
            RspCommand::InsertWatchpoint { addr, is_write } => {
                let kind = if is_write {
                    WatchpointKind::Write
                } else {
                    WatchpointKind::Read
                };
                debugger.add_watchpoint(addr, 4, kind);
                "OK".into()
            }
            RspCommand::RemoveWatchpoint { addr, .. } => {
                debugger.remove_watchpoint(addr);
                "OK".into()
            }
            RspCommand::Step => {
                debugger.request_step();
                "S05".into()
            }
            RspCommand::Continue => {
                debugger.resume();
                "OK".into()
            }
            RspCommand::Detach | RspCommand::Kill => {
                debugger.pause();
                "OK".into()
            }
            _ => String::new(),
        }
    }
}
