#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use rustc_hash::FxHashMap;
use roead::byml::Byml;
use core::mem::discriminant;
#[cfg(feature = "std")]
use thiserror::Error;
use core::hash::Hash;
use alloc::string::ToString;

#[cfg_attr(feature = "std", derive(Error))]
#[derive(Debug)]
pub enum MergeError {
    #[cfg_attr(feature = "std", error("mismatch in types (got {1:?} expected {0})"))]
    Mismatch(String, Byml)
}


fn merge_byml_hashmap<T: Hash + Eq>(base: &mut FxHashMap<T, Byml>, patch: FxHashMap<T, Byml>) -> Result<(), MergeError> {
    for (p_key, p_val) in patch.into_iter() {
        match base.get_mut(&p_key) {
            Some(b_val) => {
                merge_byml_raw(b_val, p_val)?; 
            }
            _ => {
                base.insert(p_key, p_val);
            }
        }
        
    }
    Result::Ok(())
}
fn merge_byml_value_hashmap<T: Hash + Eq>(base: &mut FxHashMap<T, (Byml, u32)>, patch: FxHashMap<T, (Byml, u32)>) -> Result<(), MergeError> {
    for (p_key, p_val) in patch.into_iter() {
        match base.get_mut(&p_key) {
            Some(b_val) => {
                merge_byml_raw(&mut b_val.0, p_val.0)?;
                b_val.1 = p_val.1;
            }
            _ => {
                base.insert(p_key, p_val);
            }
        }
    }
    Result::Ok(())
}
/// Apply `patch` to `base` in place. May cause partial patch if result is ignored.
pub fn merge_byml_raw(base: &mut Byml, patch: Byml) -> Result<(), MergeError> {
    match base {
        Byml::Map(da_map) => {
           if let Byml::Map(patch_map) = patch {
                merge_byml_hashmap(da_map, patch_map)?;
           } else {
                return Result::Err(MergeError::Mismatch("Map".to_string(), patch));
           } 
        },
        Byml::Array(arr) => {
            match patch {
                // patch on integer
                Byml::HashMap(patch_map) => {
                    for (key, value) in patch_map.into_iter() {
                        merge_byml_raw(&mut arr[key as usize], value)?;
                    }
                },
                // patch all
                Byml::Array(p_arr) => {
                    
                    for (b_item, p_item) in arr.iter_mut().zip(p_arr.into_iter()) {
                       merge_byml_raw(b_item, p_item)?;
                   }
                },
                _ => {
                    return Result::Err(MergeError::Mismatch("Array".to_string(), patch));
                }
            } 
        },
        Byml::HashMap(hashmap) => {
            if let Byml::HashMap(patch_map) = patch {
                merge_byml_hashmap(hashmap, patch_map)?;
            } else {
                return Result::Err(MergeError::Mismatch("HashMap".to_string(), patch));
            }
        },
        Byml::ValueHashMap(vhashmap) => {
            if let Byml::ValueHashMap(patch_map) = patch {
                merge_byml_value_hashmap(vhashmap, patch_map)?;
            } else {
                return Result::Err(MergeError::Mismatch("ValueHashMap".to_string(), patch));
            }
        },
        _ => {
            if discriminant(base) != discriminant(&patch) {
                return Result::Err(MergeError::Mismatch(format!("{:?}", base), patch));
            }
            *base = patch;
        }
    }
    return Result::Ok(());
}

/// Apply patches from `patch` to `base`. Moves `base` so it's never in an incomplete state.
pub fn merge_byml(base: Byml, patch: Byml) -> Result<Byml, MergeError> {
    let mut res = base;
    merge_byml_raw(&mut res, patch)?;
    Result::Ok(res)
}
