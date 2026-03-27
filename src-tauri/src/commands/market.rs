use crate::types::{
    DownloadRequest, DownloadResult, MarketStatus, MarketStatusType, RemoteSkill, RemoteSkillView,
    RemoteSkillsResponse, RemoteSkillsViewResponse,
};
use crate::utils::download::{download_bytes, download_skill_to_dir};
use std::collections::HashMap;
use std::path::PathBuf;

fn map_claude_skill(skill: RemoteSkill, market_id: &str, market_label: &str) -> RemoteSkillView {
    RemoteSkillView {
        id: format!("{}:{}", market_id, skill.id),
        name: skill.name,
        namespace: skill.namespace,
        source_url: skill.source_url,
        description: skill.description,
        author: skill.author,
        installs: skill.installs,
        stars: skill.stars,
        market_id: market_id.to_string(),
        market_label: market_label.to_string(),
    }
}

fn build_github_source_url(owner: &str, repo: &str) -> String {
    format!("https://github.com/{}/{}", owner, repo)
}

fn get_value_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(found) = value.get(*key) {
            if let Some(s) = found.as_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn get_value_u64(value: &serde_json::Value, keys: &[&str]) -> Option<u64> {
    for key in keys {
        if let Some(found) = value.get(*key) {
            if let Some(n) = found.as_u64() {
                return Some(n);
            }
            if let Some(n) = found.as_i64() {
                if n >= 0 {
                    return Some(n as u64);
                }
            }
        }
    }
    None
}

fn parse_skillsllm(
    buf: &[u8],
    market_id: &str,
    market_label: &str,
) -> Result<(Vec<RemoteSkillView>, u64), String> {
    let value: serde_json::Value = serde_json::from_slice(buf).map_err(|err| err.to_string())?;

    let list = value.get("skills").and_then(|v| v.as_array());

    let mut skills = Vec::new();
    if let Some(items) = list {
        for item in items {
            let github_owner =
                get_value_string(item, &["githubOwner", "github_owner", "owner", "repoOwner"]);
            let github_repo =
                get_value_string(item, &["githubRepo", "github_repo", "repo", "repoName"]);
            let source_url =
                get_value_string(item, &["githubUrl", "sourceUrl", "source_url", "repoUrl"])
                    .or_else(|| match (github_owner.as_deref(), github_repo.as_deref()) {
                        (Some(o), Some(r)) => Some(build_github_source_url(o, r)),
                        _ => None,
                    })
                    .unwrap_or_default();

            let name = get_value_string(item, &["name", "title"])
                .or_else(|| github_repo.clone())
                .unwrap_or_else(|| "skill".to_string());
            let description =
                get_value_string(item, &["description", "summary"]).unwrap_or_default();
            let author = get_value_string(item, &["githubOwner", "github_owner", "author"])
                .unwrap_or_default();
            let namespace = get_value_string(item, &["namespace"])
                .or_else(|| github_owner.clone())
                .unwrap_or_default();
            let stars = get_value_u64(item, &["stars", "githubStars", "github_stars"]).unwrap_or(0);
            let installs = get_value_u64(item, &["installs", "downloads"]).unwrap_or(0);
            let raw_id = get_value_string(item, &["id", "slug"])
                .or_else(|| match (github_owner.as_deref(), github_repo.as_deref()) {
                    (Some(o), Some(r)) => Some(format!("{}/{}", o, r)),
                    _ => None,
                })
                .unwrap_or_else(|| name.clone());

            skills.push(RemoteSkillView {
                id: format!("{}:{}", market_id, raw_id),
                name,
                namespace,
                source_url,
                description,
                author,
                installs,
                stars,
                market_id: market_id.to_string(),
                market_label: market_label.to_string(),
            });
        }
    }

    let total = value
        .get("pagination")
        .and_then(|p| get_value_u64(p, &["total", "count"]))
        .unwrap_or(skills.len() as u64);

    Ok((skills, total))
}

fn parse_skillsmp(
    buf: &[u8],
    market_id: &str,
    market_label: &str,
) -> Result<(Vec<RemoteSkillView>, u64), String> {
    let value: serde_json::Value = serde_json::from_slice(buf).map_err(|err| err.to_string())?;

    let list = value
        .get("data")
        .and_then(|d| d.get("skills"))
        .and_then(|v| v.as_array());

    let mut skills = Vec::new();
    if let Some(items) = list {
        for item in items {
            let source_url = get_value_string(item, &["githubUrl", "sourceUrl", "source_url"])
                .unwrap_or_default();
            let author = get_value_string(item, &["author"]).unwrap_or_default();

            let name =
                get_value_string(item, &["name", "title"]).unwrap_or_else(|| "skill".to_string());
            let description =
                get_value_string(item, &["description", "summary"]).unwrap_or_default();
            let namespace = author.clone();
            let stars = get_value_u64(item, &["stars", "githubStars", "github_stars"]).unwrap_or(0);
            let installs = get_value_u64(item, &["installs", "downloads"]).unwrap_or(0);
            let raw_id = get_value_string(item, &["id", "slug"]).unwrap_or_else(|| name.clone());

            skills.push(RemoteSkillView {
                id: format!("{}:{}", market_id, raw_id),
                name,
                namespace,
                source_url,
                description,
                author,
                installs,
                stars,
                market_id: market_id.to_string(),
                market_label: market_label.to_string(),
            });
        }
    }

    let total = value
        .get("data")
        .and_then(|d| d.get("pagination"))
        .and_then(|p| get_value_u64(p, &["total", "count"]))
        .unwrap_or(skills.len() as u64);

    Ok((skills, total))
}

#[tauri::command]
pub async fn search_marketplaces(
    query: String,
    limit: u64,
    offset: u64,
    api_keys: HashMap<String, String>,
    enabled_markets: HashMap<String, bool>,
) -> Result<RemoteSkillsViewResponse, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let trimmed = query.trim();
        let query_param = if trimmed.is_empty() {
            String::new()
        } else {
            format!("q={}", urlencoding::encode(trimmed))
        };

        let limit = if limit == 0 { 20 } else { limit };

        // Use std::thread::scope to run three marketplace searches concurrently
        let results = std::thread::scope(|scope| {
            // Prepare data for Claude Plugins marketplace
            let claude_market_id = "claude-plugins";
            let claude_market_label = "Claude Plugins";
            let claude_enabled = *enabled_markets.get(claude_market_id).unwrap_or(&true);

            let query_param_clone = query_param.clone();
            let claude_handle = scope.spawn(move || {
                let mut skills = Vec::new();
                let mut total = 0u64;
                let mut status = MarketStatus {
                    id: claude_market_id.to_string(),
                    name: claude_market_label.to_string(),
                    status: MarketStatusType::Online,
                    error: None,
                };

                if claude_enabled {
                    let mut url = String::from("https://claude-plugins.dev/api/skills?");
                    if !query_param_clone.is_empty() {
                        url.push_str(&query_param_clone);
                        url.push('&');
                    }
                    url.push_str(&format!("limit={}&offset={}", limit, offset));

                    match download_bytes(
                        &url,
                        &[
                            ("Accept", "application/json"),
                            ("User-Agent", "qing-skill-manager/0.1"),
                        ],
                    ) {
                        Ok(buf) => {
                            if let Ok(parsed) = serde_json::from_slice::<RemoteSkillsResponse>(&buf) {
                                total = parsed.total;
                                skills = parsed.skills.into_iter().map(|skill| {
                                    map_claude_skill(skill, claude_market_id, claude_market_label)
                                }).collect();
                            } else {
                                status.status = MarketStatusType::Error;
                                status.error = Some("Failed to parse response".to_string());
                            }
                        }
                        Err(e) => {
                            println!("Error fetching from Claude Plugins: {}", e);
                            status.status = MarketStatusType::Error;
                            status.error = Some(e);
                        }
                    }
                }

                (skills, total, status)
            });

            // Prepare data for SkillsLLM marketplace
            let skillsllm_market_id = "skillsllm";
            let skillsllm_market_label = "SkillsLLM";
            let skillsllm_enabled = *enabled_markets.get(skillsllm_market_id).unwrap_or(&true);

            let query_param_clone = query_param.clone();
            let skillsllm_handle = scope.spawn(move || {
                let mut skills = Vec::new();
                let mut total = 0u64;
                let mut status = MarketStatus {
                    id: skillsllm_market_id.to_string(),
                    name: skillsllm_market_label.to_string(),
                    status: MarketStatusType::Online,
                    error: None,
                };

                if skillsllm_enabled {
                    let skillsllm_page = (offset / limit).saturating_add(1);
                    let mut skillsllm_url = String::from("https://skillsllm.com/api/skills?");
                    if !query_param_clone.is_empty() {
                        skillsllm_url.push_str(&query_param_clone);
                        skillsllm_url.push('&');
                    }
                    skillsllm_url.push_str(&format!("page={}&limit={}", skillsllm_page, limit));

                    match download_bytes(
                        &skillsllm_url,
                        &[
                            ("Accept", "application/json"),
                            ("User-Agent", "qing-skill-manager/0.1"),
                        ],
                    ) {
                        Ok(buf) => {
                            if let Ok((parsed_skills, parsed_total)) =
                                parse_skillsllm(&buf, skillsllm_market_id, skillsllm_market_label)
                            {
                                total = parsed_total;
                                skills = parsed_skills;
                            } else {
                                status.status = MarketStatusType::Error;
                                status.error = Some("Failed to parse response".to_string());
                            }
                        }
                        Err(e) => {
                            println!("Error fetching from SkillsLLM: {}", e);
                            status.status = MarketStatusType::Error;
                            status.error = Some(e);
                        }
                    }
                }

                (skills, total, status)
            });

            // Prepare data for SkillsMP marketplace
            let skillsmp_market_id = "skillsmp";
            let skillsmp_market_label = "SkillsMP";
            let skillsmp_enabled = *enabled_markets.get(skillsmp_market_id).unwrap_or(&false);
            let skillsmp_api_key = api_keys.get(skillsmp_market_id).cloned();

            let skillsmp_handle = scope.spawn(move || {
                let mut skills = Vec::new();
                let mut total = 0u64;
                let mut status = MarketStatus {
                    id: skillsmp_market_id.to_string(),
                    name: skillsmp_market_label.to_string(),
                    status: if skillsmp_enabled { MarketStatusType::NeedsKey } else { MarketStatusType::Error },
                    error: if skillsmp_enabled { None } else { Some("Market is disabled".to_string()) },
                };

                if skillsmp_enabled {
                    if let Some(api_key) = skillsmp_api_key.as_ref().filter(|k| !k.is_empty()) {
                        let skillsmp_page = (offset / limit).saturating_add(1);
                        let skillsmp_url = format!(
                            "https://skillsmp.com/api/v1/skills/search?q={}&page={}&limit={}",
                            urlencoding::encode(trimmed),
                            skillsmp_page,
                            limit
                        );

                        let auth_header = format!("Bearer {}", api_key);

                        match download_bytes(
                            &skillsmp_url,
                            &[
                                ("Accept", "application/json"),
                                ("User-Agent", "qing-skill-manager/0.1"),
                                ("Authorization", &auth_header),
                            ],
                        ) {
                            Ok(buf) => {
                                if let Ok((parsed_skills, parsed_total)) =
                                    parse_skillsmp(&buf, skillsmp_market_id, skillsmp_market_label)
                                {
                                    total = parsed_total;
                                    skills = parsed_skills;
                                    status.status = MarketStatusType::Online;
                                } else {
                                    status.status = MarketStatusType::Error;
                                    status.error = Some("Failed to parse response".to_string());
                                }
                            }
                            Err(e) => {
                                println!("Error fetching from SkillsMP: {}", e);
                                status.status = MarketStatusType::Error;
                                status.error = Some(e);
                            }
                        }
                    }
                }

                (skills, total, status)
            });

            // Wait for all threads to complete and collect results
            let (claude_skills, claude_total, claude_status) = claude_handle.join().unwrap_or_else(|_| {
                (Vec::new(), 0, MarketStatus {
                    id: "claude-plugins".to_string(),
                    name: "Claude Plugins".to_string(),
                    status: MarketStatusType::Error,
                    error: Some("Thread panicked".to_string()),
                })
            });

            let (skillsllm_skills, skillsllm_total, skillsllm_status) = skillsllm_handle.join().unwrap_or_else(|_| {
                (Vec::new(), 0, MarketStatus {
                    id: "skillsllm".to_string(),
                    name: "SkillsLLM".to_string(),
                    status: MarketStatusType::Error,
                    error: Some("Thread panicked".to_string()),
                })
            });

            let (skillsmp_skills, skillsmp_total, skillsmp_status) = skillsmp_handle.join().unwrap_or_else(|_| {
                (Vec::new(), 0, MarketStatus {
                    id: "skillsmp".to_string(),
                    name: "SkillsMP".to_string(),
                    status: MarketStatusType::Error,
                    error: Some("Thread panicked".to_string()),
                })
            });

            // Merge results from all three marketplaces
            let mut all_skills = Vec::new();
            all_skills.extend(claude_skills);
            all_skills.extend(skillsllm_skills);
            all_skills.extend(skillsmp_skills);

            let total_count = claude_total + skillsllm_total + skillsmp_total;

            let market_statuses = vec![claude_status, skillsllm_status, skillsmp_status];

            (all_skills, total_count, market_statuses)
        });

        let (skills, total, market_statuses) = results;

        Ok(RemoteSkillsViewResponse {
            skills,
            total,
            limit,
            offset,
            market_statuses,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn download_marketplace_skill(
    request: DownloadRequest,
) -> Result<DownloadResult, String> {
    if request.install_base_dir.trim().is_empty() {
        return Err("Install directory cannot be empty".to_string());
    }

    let source_url = request.source_url.clone();
    let skill_name = request.skill_name.clone();
    let install_base_dir = PathBuf::from(&request.install_base_dir);

    let result = tauri::async_runtime::spawn_blocking(move || {
        download_skill_to_dir(&source_url, &skill_name, &install_base_dir, false)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(DownloadResult {
        installed_path: result.display().to_string(),
    })
}

#[tauri::command]
pub async fn update_marketplace_skill(request: DownloadRequest) -> Result<DownloadResult, String> {
    if request.install_base_dir.trim().is_empty() {
        return Err("Install directory cannot be empty".to_string());
    }
    if request.source_url.trim().is_empty() {
        return Err("Missing valid source URL, cannot update".to_string());
    }

    let source_url = request.source_url.clone();
    let skill_name = request.skill_name.clone();
    let install_base_dir = PathBuf::from(&request.install_base_dir);

    let result = tauri::async_runtime::spawn_blocking(move || {
        download_skill_to_dir(&source_url, &skill_name, &install_base_dir, true)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(DownloadResult {
        installed_path: result.display().to_string(),
    })
}
