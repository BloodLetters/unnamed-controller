/// Defines different regions of memory for an MCU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryRegionType {
    Flash,
    Sram,
    Eeprom,
    Register,
}

/// Represents a contiguous block of memory with a specific base address.
#[derive(Clone)]
pub struct MemoryRegion {
    pub name: String,
    pub region_type: MemoryRegionType,
    pub base_address: u32,
    pub size: usize,
    pub data: Vec<u8>,
}

impl MemoryRegion {
    /// Creates a new memory region initialized to zeros.
    pub fn new(name: &str, region_type: MemoryRegionType, base_address: u32, size: usize) -> Self {
        Self {
            name: name.to_string(),
            region_type,
            base_address,
            size,
            data: vec![0; size],
        }
    }

    /// Checks if a global address falls within this region.
    pub fn contains(&self, address: u32) -> bool {
        address >= self.base_address && (address - self.base_address) < self.size as u32
    }

    /// Reads a byte using a global address.
    pub fn read_byte(&self, address: u32) -> Option<u8> {
        if self.contains(address) {
            let offset = (address - self.base_address) as usize;
            Some(self.data[offset])
        } else {
            None
        }
    }

    /// Writes a byte using a global address.
    pub fn write_byte(&mut self, address: u32, value: u8) -> bool {
        if self.contains(address) {
            let offset = (address - self.base_address) as usize;
            self.data[offset] = value;
            true
        } else {
            false
        }
    }
}

/// Manages multiple memory regions, mapping global addresses to specific regions.
#[derive(Clone)]
pub struct MemoryMapper {
    pub regions: Vec<MemoryRegion>,
}

impl Default for MemoryMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryMapper {
    /// Creates a new empty memory mapper.
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    /// Adds a new memory region to the mapper.
    pub fn add_region(&mut self, region: MemoryRegion) {
        self.regions.push(region);
    }

    /// Inserts a region, replacing any existing region that shares its name.
    pub fn set_region(&mut self, region: MemoryRegion) {
        if let Some(existing) = self
            .regions
            .iter_mut()
            .find(|candidate| candidate.name == region.name)
        {
            *existing = region;
        } else {
            self.regions.push(region);
        }
    }

    /// Reads a byte from the appropriate memory region.
    pub fn read_byte(&self, address: u32) -> Option<u8> {
        for region in &self.regions {
            if let Some(val) = region.read_byte(address) {
                return Some(val);
            }
        }
        None
    }

    /// Writes a byte to the appropriate memory region.
    pub fn write_byte(&mut self, address: u32, value: u8) -> bool {
        for region in &mut self.regions {
            if region.write_byte(address, value) {
                return true;
            }
        }
        false
    }

    /// Clears (zeroes) all memory regions.
    pub fn clear(&mut self) {
        for region in &mut self.regions {
            region.data.fill(0);
        }
    }
}
