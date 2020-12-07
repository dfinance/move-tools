use http::Uri;
use libra::prelude::*;
use anyhow::Result;
use move_core_types::language_storage::StructTag;
use serde::{Serialize, Deserialize};

pub fn get_module<'a, T: Into<&'a Uri>>(module_id: &ModuleId, url: T) -> Result<Vec<u8>> {
    let path = AccessPath::code_access_path(module_id);
    let url = format!(
        "{base_url}vm/data/{address}/{path}",
        base_url = url.into(),
        address = hex::encode(&path.address),
        path = hex::encode(path.path)
    );

    let resp = reqwest::blocking::get(&url)?;
    if resp.status().is_success() {
        let res: LoaderResponse = resp.json()?;
        Ok(hex::decode(&res.result.value)?)
    } else {
        let res: LoaderErrorResponse = resp.json()?;
        Err(anyhow!(
            "Failed to load dependencies :'{}' [{}]",
            url,
            res.error
        ))
    }
}

pub fn get_resource<'a, T: Into<&'a Uri>>(res: &ResourceKey, url: T) -> Result<Vec<u8>> {
    let path = AccessPath::resource_access_path(res);
    let url = format!(
        "{base_url}vm/data/{address}/{path}",
        base_url = url.into(),
        address = hex::encode(&path.address),
        path = hex::encode(path.path)
    );

    trace!("url: {:?}", url);
    let resp = reqwest::blocking::get(&url)?;
    let status = resp.status();

    if status.is_success() {
        let body = resp.text()?;
        trace!("response: ({}) {}", status, body);
        let res: LoaderResponse = serde_json::from_str(&body)?;
        Ok(hex::decode(&res.result.value)?)
    } else {
        let res: LoaderErrorResponse = resp.json()?;
        Err(anyhow!("Failed to load resource:'{}' [{}]", url, res.error))
    }
}

pub struct DnodeRestClient {
    uri: Uri,
}

impl DnodeRestClient {
    pub fn new<T: Into<Uri>>(uri: T) -> Self {
        Self { uri: uri.into() }
    }
}

impl RemoteCache for DnodeRestClient {
    fn get_module(&self, id: &ModuleId) -> VMResult<Option<Vec<u8>>> {
        debug!("req: (mod) {:?}", id);
        let res = get_module(id, &self.uri).ok();
        if res.is_some() && res.as_ref().unwrap().len() == 0 {
            error!("Err: empty module for {}", id);
        }
        Ok(res)
    }

    fn get_resource(
        &self,
        addr: &AccountAddress,
        tag: &StructTag,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        debug!(
            "req: (res) address: {}, tag: {}",
            addr.to_string(),
            tag.to_string()
        );
        let key = ResourceKey::new(*addr, tag.to_owned());
        let res = get_resource(&key, &self.uri).ok();
        if res.is_some() && res.as_ref().unwrap().len() == 0 {
            error!("Err: empty module for {:?}", key);
        }
        Ok(res)
    }
}

/// Api response.
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct LoaderResponse {
    /// Result.
    pub result: Response,
}

/// Success response.
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Response {
    /// Hex encoded bytecode.
    pub value: String,
}

///Error response.
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct LoaderErrorResponse {
    /// Error message.
    pub error: String,
}
