use super::*;
use std::sync::Arc;
use crate::sys;

/// Represents the result of an loaditem operation, ready to be processed.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamInventoryDefinitionUpdate { }

unsafe impl Callback for SteamInventoryDefinitionUpdate {
    const ID: i32 = sys::SteamInventoryDefinitionUpdate_t_k_iCallback as i32;
    const SIZE: i32 = std::mem::size_of::<sys::SteamInventoryDefinitionUpdate_t>() as i32;

    unsafe fn from_raw(_raw: *mut c_void) -> Self {
        Self { }
    }
}

/// Represents the result of an inventory operation, ready to be processed.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamInventoryResultReady {
    pub handle: sys::SteamInventoryResult_t,
    pub result: Result<(), SteamError>,
}

unsafe impl Callback for SteamInventoryResultReady {
    const ID: i32 = sys::SteamInventoryResultReady_t_k_iCallback as i32;
    const SIZE: i32 = std::mem::size_of::<sys::SteamInventoryResultReady_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let status = &*(raw as *mut sys::SteamInventoryResultReady_t);
        Self {
            handle: status.m_handle,
            result: match status.m_result {
                sys::EResult::k_EResultOK => Ok(()),
                _ => Err(SteamError::from(status.m_result)),
            },
        }
    }
}

/// Represents a full update event for the Steam inventory.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SteamInventoryFullUpdate {
    pub handle: sys::SteamInventoryResult_t,
}

unsafe impl Callback for SteamInventoryFullUpdate {
    const ID: i32 = sys::SteamInventoryFullUpdate_t_k_iCallback as i32;
    const SIZE: i32 = std::mem::size_of::<sys::SteamInventoryFullUpdate_t>() as i32;

    unsafe fn from_raw(raw: *mut c_void) -> Self {
        let status = &*(raw as *mut sys::SteamInventoryFullUpdate_t);
        Self {
            handle: status.m_handle,
        }
    }
}

/// Provides access to the Steam inventory interface.
pub struct Inventory<Manager> {
    pub(crate) inventory: *mut sys::ISteamInventory,
    pub(crate) _inner: Arc<Inner<Manager>>,
}

impl<Manager> Inventory<Manager> {
    /// Load item definitions from Steam.
    pub fn load_item_definitions(&self) -> Result<(), InventoryError> {
        unsafe {
            if sys::SteamAPI_ISteamInventory_LoadItemDefinitions(self.inventory) {
                Ok(())
            } else {
                Err(InventoryError::LoadItemDefinitionsFailed)
            }
        }
    }

    pub fn get_item_definitions_ids(&self) -> Result<Vec<sys::SteamItemDef_t>, InventoryError> {
        let mut item_defs_count = 0;
        unsafe {
            if !sys::SteamAPI_ISteamInventory_GetItemDefinitionIDs(
                self.inventory,
                std::ptr::null_mut(),
                &mut item_defs_count,
            ) {
                return Err(InventoryError::GetItemDefinitionIDsFailed);
            }

            let mut item_defs_array: Vec<sys::SteamItemDef_t> = Vec::with_capacity(item_defs_count as usize);
            if sys::SteamAPI_ISteamInventory_GetItemDefinitionIDs(
                self.inventory,
                item_defs_array.as_mut_ptr(),
                &mut item_defs_count,
            ) {
                item_defs_array.set_len(item_defs_count as usize);
                Ok(item_defs_array)
            } else {
                Err(InventoryError::GetItemDefinitionIDsFailed)
            }
        }
    }

    pub fn get_item_definition_property(&self, item_def: sys::SteamItemDef_t, property_name: &str) -> Result<String, InventoryError> {
        let property_name = CString::new(property_name).expect("CString::new failed");
        let mut value_len = 0;
        unsafe {
            if !sys::SteamAPI_ISteamInventory_GetItemDefinitionProperty(
                self.inventory,
                item_def,
                property_name.as_ptr(),
                std::ptr::null_mut(),
                &mut value_len,
            ) {
                return Err(InventoryError::GetItemDefinitionPropertyFailed);
            }
            
            let mut value_buffer: Vec<u8> = Vec::with_capacity(value_len as usize);
            if sys::SteamAPI_ISteamInventory_GetItemDefinitionProperty(
                self.inventory,
                item_def,
                property_name.as_ptr(),
                value_buffer.as_mut_ptr() as *mut i8,
                &mut value_len,
            ) {
                value_buffer.set_len((value_len - 1) as usize);
                let value = String::from_utf8(value_buffer).expect("Failed to convert value to string");
                Ok(value)
            } else {
                Err(InventoryError::GetItemDefinitionPropertyFailed)
            }
        }
    }

    pub fn trigger_item_drop(&self, drop_list_definition: sys::SteamItemDef_t) -> Result<sys::SteamInventoryResult_t, InventoryError> {
        let mut result_handle = sys::k_SteamInventoryResultInvalid;
        unsafe {
            if sys::SteamAPI_ISteamInventory_TriggerItemDrop(self.inventory, &mut result_handle, drop_list_definition) {
                Ok(result_handle)
            } else {
                Err(InventoryError::TriggerItemDropFailed)
            }
        }
    }

    pub fn consume_item(&self, item_consume: sys::SteamItemInstanceID_t, quantity: u32) -> Result<sys::SteamInventoryResult_t, InventoryError> {
        let mut result_handle = sys::k_SteamInventoryResultInvalid;
        unsafe {
            if sys::SteamAPI_ISteamInventory_ConsumeItem(self.inventory, &mut result_handle, item_consume, quantity) {
                Ok(result_handle)
            } else {
                Err(InventoryError::OperationFailed)
            }
        }
    }

    /// Retrieves all items in the user's Steam inventory.
    pub fn get_all_items(&self) -> Result<sys::SteamInventoryResult_t, InventoryError> {
        let mut result_handle = sys::k_SteamInventoryResultInvalid;
        unsafe {
            if sys::SteamAPI_ISteamInventory_GetAllItems(self.inventory, &mut result_handle) {
                Ok(result_handle)
            } else {
                Err(InventoryError::OperationFailed)
            }
        }
    }

    /// Retrieves the status of a result handle.
    pub fn get_result_status(&self, result_handle: sys::SteamInventoryResult_t) -> Result<sys::EResult, InventoryError> {
        unsafe {
            let status = sys::SteamAPI_ISteamInventory_GetResultStatus(
                self.inventory,
                result_handle,
            );
            if status == sys::EResult::k_EResultOK {
                Ok(status)
            } else {
                Err(InventoryError::GetResultStatusFailed)
            }
        }
    }

    /// Retrieves the detailed list of items from the inventory given a result handle.
    pub fn get_result_items(&self, result_handle: sys::SteamInventoryResult_t) -> Result<Vec<SteamItemDetails>, InventoryError> {
        let mut items_count = 0;
        unsafe {
            if !sys::SteamAPI_ISteamInventory_GetResultItems(
                self.inventory,
                result_handle,
                std::ptr::null_mut(),
                &mut items_count,
            ) {
                return Err(InventoryError::GetResultItemsFailed);
            }

            let mut items_array: Vec<sys::SteamItemDetails_t> = Vec::with_capacity(items_count as usize);
            if sys::SteamAPI_ISteamInventory_GetResultItems(
                self.inventory,
                result_handle,
                items_array.as_mut_ptr(),
                &mut items_count,
            ) {
                items_array.set_len(items_count as usize);
                let items = items_array.into_iter().map(|details| SteamItemDetails {
                    item_id: SteamItemInstanceID(details.m_itemId),
                    definition: SteamItemDef(details.m_iDefinition),
                    quantity: details.m_unQuantity,
                    flags: details.m_unFlags,
                }).collect();
                Ok(items)
            } else {
                Err(InventoryError::GetResultItemsFailed)
            }
        }
    }

    /// Destroy a result handle after use.
    pub fn destroy_result(&self, result_handle: sys::SteamInventoryResult_t) {
        unsafe {
            sys::SteamAPI_ISteamInventory_DestroyResult(
                self.inventory,
                result_handle,
            );
        }
    }
}

/// Represents an individual inventory item with its unique details.
#[derive(Clone, Debug)]
pub struct SteamItemDetails {
    pub item_id: SteamItemInstanceID,
    pub definition: SteamItemDef,
    pub quantity: u16,
    pub flags: u16,
}

/// Represents a unique identifier for an inventory item instance.
#[derive(Clone, Debug)]
pub struct SteamItemInstanceID(pub u64);

/// Represents a unique identifier for an item definition.
#[derive(Clone, Debug)]
pub struct SteamItemDef(pub i32);

/// Enumerates possible errors that can occur during inventory operations.
#[derive(Debug, Error)]
pub enum InventoryError {
    #[error("The inventory operation failed")]
    OperationFailed,
    #[error("Failed to trigger item drop")]
    TriggerItemDropFailed,
    #[error("Failed to consume item")]
    ConsumeItemFailed,
    #[error("Failed to retrieve result status")]
    GetResultStatusFailed,
    #[error("Failed to retrieve result items")]
    GetResultItemsFailed,
    #[error("Invalid input")]
    InvalidInput,
    #[error("Load item definitions failed")]
    LoadItemDefinitionsFailed,
    #[error("Failed to retrieve item definition IDs")]
    GetItemDefinitionIDsFailed,
    #[error("Failed to retrieve item definition property")]
    GetItemDefinitionPropertyFailed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn test_get_result_items() {
        let client = Client::init().unwrap();
        let (tx, rx) = mpsc::channel::<sys::SteamInventoryResult_t>();

        client.register_callback(move |val: SteamInventoryResultReady| {
            assert!(val.result.is_ok(), "SteamInventoryResultReady Failed.");
            if let Ok(_) = val.result {
                tx.send(val.handle).expect("Failed to send handle");
            }
        });

        client.register_callback(move |val: SteamInventoryFullUpdate| {
            println!("SteamInventoryFullUpdate: {:?}", val)
        });

        let _result = client.inventory().get_all_items();

        for _ in 0..50 {
            client.run_callbacks();
            ::std::thread::sleep(::std::time::Duration::from_millis(100));
            if let Ok(handle) = rx.try_recv() {
                let result_items = client.inventory().get_result_items(handle).unwrap();
                assert!(!result_items.is_empty(), "No items received");
                println!("Result items: {:?}", result_items);
                client.inventory().destroy_result(handle);
                return;
            }
        }
        panic!("Timed out waiting for inventory result.");
    }
}