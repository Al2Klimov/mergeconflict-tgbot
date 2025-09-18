use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Root {
    pub(crate) data: Data,
}

#[derive(Deserialize)]
pub(crate) struct Data {
    pub(crate) viewer: Viewer,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Viewer {
    pub(crate) pull_requests: PullRequests,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PullRequests {
    pub(crate) nodes: Vec<Node>,
    pub(crate) page_info: PageInfo,
}

#[derive(Deserialize)]
pub(crate) struct Node {
    pub(crate) url: String,
    pub(crate) mergeable: Mergeable,
    pub(crate) title: String,
}

#[derive(Deserialize)]
pub(crate) enum Mergeable {
    UNKNOWN,
    MERGEABLE,
    CONFLICTING,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PageInfo {
    pub(crate) has_next_page: bool,
    pub(crate) end_cursor: Option<String>,
}
