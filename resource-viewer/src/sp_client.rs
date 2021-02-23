use std::cell::RefCell;
use std::rc::Rc;
use log::{error, info, debug};
use keyring::sr25519::sr25519::Pair;
use libra::move_core_types::language_storage::StructTag;
use substrate_api_client::Api;
use libra::prelude::*;
use anyhow::{Error, Result};
use http::Uri;

/// Block number
pub type Block = u128;
pub struct BytesForBlock(Vec<u8>, Block);

#[allow(dead_code)]
impl BytesForBlock {
    pub fn block(&self) -> u128 {
        self.1
    }

    #[inline]
    pub fn bytes(&self) -> &[u8] {
        self.as_bytes()
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

pub fn get_resource(
    key: &ResourceKey,
    host: &Uri,
    height: Option<Block>,
) -> Result<BytesForBlock> {
    let mut client = Api::new(host.to_string());
    get_resource_with(&mut client, key, height)
}

#[allow(dead_code)]
pub fn get_module(
    module_id: &ModuleId,
    host: Uri,
    height: Option<Block>,
) -> Result<BytesForBlock> {
    let mut client = Api::new(host.to_string());
    get_module_with(&mut client, module_id, height)
}

pub fn get_module_with(
    client: &mut Api<Pair>,
    module_id: &ModuleId,
    height: Option<Block>,
) -> Result<BytesForBlock> {
    // same as AccessPath::code_access_path(module_id)
    let path = module_id.access_vector();
    data_request_with(client, path, height)
}

const MODULE: &str = "Mvm";

pub fn data_request_with(
    client: &mut Api<Pair>,
    path: Vec<u8>,
    _height: Option<Block>,
) -> Result<BytesForBlock> {
    let storagekey = client
        .metadata
        .storage_map_key::<Vec<u8>, Vec<u8>>(MODULE, "VMStorage", path)
        .unwrap();
    debug!("storagekey {:?}", storagekey);
    debug!("storage key is: 0x{}", hex::encode(storagekey.0.clone()));
    let result: Option<Vec<u8>> = client.get_storage_by_key_hash(storagekey, None);
    info!("data: {:?}", result);

    result
        .ok_or_else(|| Error::msg("not found"))
        .map(|result| BytesForBlock(result, 0))
}

pub fn get_resource_with(
    client: &mut Api<Pair>,
    key: &ResourceKey,
    _height: Option<Block>,
) -> Result<BytesForBlock> {
    let path = AccessPath::resource_access_path(key);
    let res_key = path.path;

    let storagekey = client
        .metadata
        .storage_map_key::<Vec<u8>, Vec<u8>>(MODULE, "VMStorage", res_key)
        .unwrap();
    debug!("storagekey {:?}", storagekey);
    debug!("storage key is: 0x{}", hex::encode(storagekey.0.clone()));
    let result: Option<Vec<u8>> = client.get_storage_by_key_hash(storagekey, None);
    info!("data: {:?}", result);

    result
        .ok_or_else(|| Error::msg("not found"))
        .map(|result| BytesForBlock(result, 0))
}

pub struct NodeClient {
    client: Rc<RefCell<Api<Pair>>>,
    height: Option<Block>,
}

impl NodeClient {
    pub fn new<T: Into<Uri>>(uri: T, height: Option<Block>) -> Self {
        Self {
            client: Rc::new(RefCell::new(Api::new(uri.into().to_string()))),
            height,
        }
    }
}

impl RemoteCache for NodeClient {
    fn get_module(&self, id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        let res = get_module_with(&mut self.client.borrow_mut(), id, self.height)
            .map(|v| v.0)
            .ok();
        if res.is_some() && res.as_ref().unwrap().is_empty() {
            error!("Empty module for {}", id);
        }
        Ok(res)
    }

    fn get_resource(
        &self,
        addr: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        let key = ResourceKey::new(*addr, tag.to_owned());
        let res = get_resource_with(&mut self.client.borrow_mut(), &key, self.height)
            .map(|v| v.0)
            .ok();
        if res.is_some() && res.as_ref().unwrap().is_empty() {
            error!("Empty resource for {:?}", key);
        }
        Ok(res)
    }
}
