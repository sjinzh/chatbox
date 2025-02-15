use crate::slint_generatedAppWindow::ChatItem;
use std::convert::From;

#[derive(Default, Clone, Debug)]
pub struct StreamTextItem {
    pub text: Option<String>,
    pub etext: Option<String>,
    pub uuid: String,
    pub suuid: String,
    pub finished: bool,
}

#[derive(Default, Clone, Debug)]
pub struct HistoryChatItem {
    pub utext: String,
    pub btext: String,
    pub uuid: String,
}

#[derive(Default, Clone, Debug)]
pub struct HistoryChat {
    pub items: Vec<HistoryChatItem>,
}

impl From<&Vec<ChatItem>> for HistoryChat {
    fn from(item: &Vec<ChatItem>) -> Self {
        HistoryChat {
            items: item
                .iter()
                .map(|x| HistoryChatItem {
                    utext: x.utext.to_string(),
                    btext: x.btext.to_string(),
                    uuid: x.uuid.to_string(),
                })
                .collect(),
        }
    }
}


pub struct SendTextConfig {
    pub suuid: String,
    pub system_prompt: String,
    pub api_model: String,
    pub use_history: bool,
}
