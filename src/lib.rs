use reqwest::RequestBuilder;
use serde_json::{Value, json};
use std::io::Write; 

/// Struct storing your meilisearch credentials
pub struct Credentials {
    host: String,
    master_key: String,
}
impl Credentials {
    pub fn new<S: Into<String>>(host: S, master_key: S) -> Self {
        Self {
            host: host.into(),
            master_key: master_key.into(),
        }
    }
    pub fn build_request(&self, index_id: u16) -> RequestBuilder {
        let index_endpoint = format!("{}/indexes/{}/documents", &self.host, index_id);

        reqwest::Client::new()
            .post(index_endpoint)
            // .bearer_auth(&self.master_key)
    }
}

/// Struct bootstrapping json formatting layer to send structured records to MS
pub struct MeiliWriter {
    creds: Credentials,
    index: u16,
    curr_id: u16,
}

impl MeiliWriter {
    pub fn new(index: u16, creds: Credentials, curr_id: u16) -> Self {
        Self {
            index,
            creds,
            curr_id
        }
    }
}

impl Write for MeiliWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        let mut json: Value = serde_json::from_str(&s).unwrap();

        // add `id` field to entry
        if let Some(obj) = json.as_object_mut(){
            obj.insert("id".into(), json!(self.curr_id));
        }
        // dbg!("{:?}", &json);

        // spawn off task to notify meili
        let req = self.creds.build_request(self.index).json(&json);
        tokio::spawn(async move {
            if let Err(_) = req.send().await {
                println!("handle properly in real code");
            }
        });

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!()
    }
}
