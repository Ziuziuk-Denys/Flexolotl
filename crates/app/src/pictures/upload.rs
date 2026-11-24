use super::progress::{CountedBody, Progress};
use flexolotl_core::media::MediaKind;
use futures::AsyncReadExt;
use http_client::{AsyncBody, HttpClient, Method, Request};
use std::sync::Arc;

const ENDPOINT: &str = "https://catbox.moe/user/api.php";
const PUBLIC_PREFIX: &str = "https://files.catbox.moe/";
const BOUNDARY: &str = "flexolotl-7f3a9c51e2d84b06";
const MAX_RESPONSE: u64 = 1024;

#[derive(Clone)]
pub struct Upload {
    body: Arc<Vec<u8>>,
}

impl Upload {
    pub fn new(bytes: Vec<u8>, kind: MediaKind) -> Self {
        let mut body = format!(
            "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"reqtype\"\r\n\r\nfileupload\r\n\
             --{BOUNDARY}\r\nContent-Disposition: form-data; name=\"fileToUpload\"; \
             filename=\"picture.{}\"\r\nContent-Type: {}\r\n\r\n",
            kind.extension(),
            kind.mime()
        )
        .into_bytes();
        body.extend(bytes);
        body.extend(format!("\r\n--{BOUNDARY}--\r\n").into_bytes());
        Self {
            body: Arc::new(body),
        }
    }

    pub fn size(&self) -> usize {
        self.body.len()
    }

    pub async fn send(self, client: Arc<dyn HttpClient>, progress: Progress) -> Option<String> {
        self.send_to(ENDPOINT, client, progress).await
    }

    async fn send_to(
        self,
        endpoint: &str,
        client: Arc<dyn HttpClient>,
        progress: Progress,
    ) -> Option<String> {
        let size = self.size();
        let request = Request::builder()
            .method(Method::POST)
            .uri(endpoint)
            .header(
                "Content-Type",
                format!("multipart/form-data; boundary={BOUNDARY}"),
            )
            .header("Content-Length", size)
            .body(AsyncBody::from_reader(CountedBody::new(
                self.body, progress,
            )))
            .ok()?;
        let response = client.send(request).await.ok()?;
        if !response.status().is_success() {
            return None;
        }
        let mut answer = String::new();
        response
            .into_body()
            .take(MAX_RESPONSE)
            .read_to_string(&mut answer)
            .await
            .ok()?;
        public_link(&answer)
    }
}

fn public_link(answer: &str) -> Option<String> {
    let name = answer.trim().strip_prefix(PUBLIC_PREFIX)?;
    let is_plain = !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '.');
    is_plain.then(|| format!("{PUBLIC_PREFIX}{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest_client::ReqwestClient;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    fn serve_once(answer: &'static str) -> (String, thread::JoinHandle<Vec<u8>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = format!("http://{}/", listener.local_addr().unwrap());
        let server = thread::spawn(move || {
            let mut stream = listener.accept().unwrap().0;
            let mut received = Vec::new();
            let mut buffer = [0; 4096];
            let end = format!("--{BOUNDARY}--\r\n").into_bytes();
            while !received.ends_with(&end) {
                let count = stream.read(&mut buffer).unwrap();
                received.extend_from_slice(&buffer[..count]);
            }
            let length = answer.len();
            let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\r\n{answer}");
            stream.write_all(response.as_bytes()).unwrap();
            received
        });
        (address, server)
    }

    fn upload_and_get(answer: &'static str) -> (Option<String>, Vec<u8>) {
        let progress = Progress::of(1);
        let (address, server) = serve_once(answer);
        let client = Arc::new(ReqwestClient::user_agent("test").unwrap());
        let upload = Upload::new(b"pixels".to_vec(), MediaKind::Gif);
        let link = futures::executor::block_on(upload.send_to(&address, client, progress));
        (link, server.join().unwrap())
    }

    #[test]
    fn sends_the_picture_and_reads_the_link() {
        let (link, received) = upload_and_get("https://files.catbox.moe/ab12cd.gif");
        assert_eq!(link.as_deref(), Some("https://files.catbox.moe/ab12cd.gif"));
        let received = String::from_utf8_lossy(&received);
        assert!(received.contains("fileupload") && received.contains("pixels"));
        assert!(received.contains("Content-Type: image/gif"));
        assert!(received.to_lowercase().contains("content-length: "));
    }

    #[test]
    fn does_not_believe_a_strange_answer() {
        assert_eq!(upload_and_get("https://evil.example/a.gif").0, None);
    }

    #[test]
    fn trusts_only_the_expected_answer() {
        let good = "https://files.catbox.moe/ab12cd.png\n";
        assert_eq!(public_link(good).as_deref(), Some(good.trim()));
        for bad in [
            "",
            "Something went wrong",
            "https://files.catbox.moe/",
            "https://files.catbox.moe/../evil",
            "https://evil.example/ab.png",
        ] {
            assert_eq!(public_link(bad), None, "{bad}");
        }
    }

    #[test]
    fn never_sends_the_name_of_the_file() {
        let upload = Upload::new(b"pixels".to_vec(), MediaKind::Png);
        let body = String::from_utf8_lossy(&upload.body).into_owned();
        assert!(body.contains("filename=\"picture.png\""));
        assert!(body.contains("pixels\r\n--"));
    }
}
