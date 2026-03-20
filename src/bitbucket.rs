use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Page<T> {
    pub values: Vec<T>,
    pub size: Option<usize>,
    pub pagelen: Option<usize>,
    pub page: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct Pipeline {
    pub uuid: String,
    pub build_number: i32,
    pub state: State,
    #[allow(dead_code)]
    pub created_on: String,
}

#[derive(Debug, Deserialize)]
pub struct Step {
    pub uuid: String,
    pub name: String,
    pub state: State,
}

#[derive(Debug, Deserialize)]
pub struct State {
    #[allow(dead_code)]
    pub name: String, // "COMPLETED", "IN_PROGRESS", "PENDING"
    pub result: Option<ResultState>,
}

#[derive(Debug, Deserialize)]
pub struct ResultState {
    pub name: String, // "SUCCESSFUL", "FAILED", "STOPPED"
}

pub struct BitbucketClient {
    client: Client,
    base_url: String,
    username: String,
    app_password: String,
}

impl BitbucketClient {
    pub fn new(username: String, app_password: String) -> Self {
        Self {
            client: Client::new(),
            base_url: "https://api.bitbucket.org/2.0".to_string(),
            username,
            app_password,
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .basic_auth(&self.username, Some(&self.app_password))
            .send()
            .await
            .context("Failed to send request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Request failed: {} - {}", status, text);
        }

        resp.json::<T>()
            .await
            .context("Failed to parse JSON response")
    }

    pub async fn get_pipelines(&self, workspace: &str, repo: &str) -> Result<Page<Pipeline>> {
        // Sort by -created_on to get latest first
        let path = format!(
            "/repositories/{}/{}/pipelines/?sort=-created_on",
            workspace, repo
        );
        self.get(&path).await
    }

    pub async fn get_steps(
        &self,
        workspace: &str,
        repo: &str,
        pipeline_uuid: &str,
        page: Option<usize>,
    ) -> Result<Page<Step>> {
        let path: String;
        if let Some(p) = page {
            path = format!(
                "/repositories/{}/{}/pipelines/{}/steps/?page={}&pagelen=100",
                workspace, repo, pipeline_uuid, p
            );
        } else {
            path = format!(
                "/repositories/{}/{}/pipelines/{}/steps?pagelen=100",
                workspace, repo, pipeline_uuid
            );
        }
        self.get(&path).await
    }

    pub async fn get_all_steps(
        &self,
        workspace: &str,
        repo: &str,
        pipeline_uuid: &str,
    ) -> Result<Vec<Step>> {
        let mut all_steps = Vec::new();
        let mut current_page = 1;

        loop {
            let page = self
                .get_steps(workspace, repo, pipeline_uuid, Some(current_page))
                .await?;
            
            if page.values.is_empty() {
                break;
            }

            let num_on_page = page.values.len();
            all_steps.extend(page.values);

            if let Some(total_size) = page.size {
                if all_steps.len() >= total_size {
                    break;
                }
            } else if num_on_page == 0 {
                break;
            }

            current_page += 1;
        }

        Ok(all_steps)
    }

    pub async fn get_step_log(
        &self,
        workspace: &str,
        repo: &str,
        pipeline_uuid: &str,
        step_uuid: &str,
        range: Option<String>,
    ) -> Result<String> {
        let path = format!(
            "/repositories/{}/{}/pipelines/{}/steps/{}/log",
            workspace, repo, pipeline_uuid, step_uuid
        );
        let url = format!("{}{}", self.base_url, path);

        let mut req = self
            .client
            .get(&url)
            .basic_auth(&self.username, Some(&self.app_password));

        if let Some(r) = range {
            req = req.header("Range", format!("bytes={}", r));
        }

        let resp = req.send().await.context("Failed to fetch log")?;

        if !resp.status().is_success() && resp.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Log fetch failed: {} - {}", status, text);
        }

        resp.text().await.context("Failed to read log text")
    }
}
