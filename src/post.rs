use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_yaml;

use crate::error::{Error, Result};
use crate::utils::markdown_to_html;

/// blog post headers
///
/// the blog post headers is parsed using yaml format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostHeaders {
    /// post created local time, `created: 1970-01-01T00:00:00+08:00`
    pub created: DateTime<Local>,
    /// post hidden flag, `hidden: true`, default `false`
    #[serde(default)]
    pub hidden: bool,
    /// post tags, `tags: [hello, world]`, default `[]`
    #[serde(default)]
    pub tags: Vec<String>,
    /// post description
    #[serde(default)]
    pub description: String,
    /// post title
    #[serde(default)]
    pub title: String,
}

/// blog post
///
/// every blog post is composed of `head` part and `body` part.
/// the two part is separated by the first blank line.
#[derive(Serialize)]
pub struct Post {
    /// blog root path
    root: PathBuf,
    /// post path from relative root directory
    pub path: PathBuf,
    /// the post title
    pub title: String,
    /// the post url
    pub url: PathBuf,
    /// post headers
    pub headers: PostHeaders,
    /// post html body
    pub content: String,
}

impl Post {
    /// create new `Post`
    pub fn new<P: AsRef<Path>>(root: P, path: P) -> Result<Post> {
        let root = root.as_ref();
        let path = path.as_ref();
        debug!("loading post: {}", path.display());

        let (headers, content) = Self::split_file(root, path)?;
        let title = if headers.title.is_empty() {
            path.file_stem()
                .and_then(|x| x.to_str())
                .expect(&format!("post filename format error: {}", path.display()))
        } else {
            headers.title.as_ref()
        };
        let url = Path::new("/").join(path).with_extension("html");

        Ok(Post {
            root: root.to_owned(),
            path: path.to_owned(),
            title: title.to_owned(),
            url,
            headers,
            content,
        })
    }

    /// split a post into `headers` and `content`
    fn split_file(root: &Path, path: &Path) -> Result<(PostHeaders, String)> {
        let fp = root.join(path);
        let mut fo = File::open(fp)?;
        let mut content = String::new();
        fo.read_to_string(&mut content)?;

        let v: Vec<&str> = content.splitn(3, "---").collect();
        if v.len() != 3 {
            return Err(Error::PostOnlyOnePart(path.into()));
        }
        let head = v[1].trim();
        let body = v[2].trim();
        if head.is_empty() {
            return Err(Error::PostNoHead(path.into()));
        }
        if body.is_empty() {
            return Err(Error::PostNoBody(path.into()));
        }
        let mut headers: PostHeaders = match serde_yaml::from_str(head) {
            Ok(headers) => headers,
            Err(e) => {
                return Err(Error::PostHeadPaser(e, path.into()));
            }
        };
        if headers.description.is_empty() {
            let desc = body
                .split("\n\n")
                .take(1)
                .next()
                .unwrap_or("")
                .split_whitespace()
                .take(100)
                .collect::<Vec<_>>()
                .join(" ");
            headers.description.push_str(&desc);
            if !headers.description.is_empty() {
                headers.description.push_str("...");
            }
        }
        let content = markdown_to_html(&body);
        Ok((headers, content.to_string()))
    }

    /// the absolute path of blog post markdown file.
    pub fn src(&self) -> PathBuf {
        self.root.join(&self.path)
    }

    /// the absolute path of blog post html file.
    pub fn dest(&self) -> PathBuf {
        self.path.with_extension("html")
    }
}
