use async_openai::{config::OpenAIConfig, Client};
use url::Url;

use super::{
    chat::ChatClient,
    delegate::{LlmClient, OpenAiDelegate},
    embedding::EmbeddingClient,
    instruct::InstructClient,
};

pub(crate) enum OpenAiDelegateBuilderArgument {
    Endpoint(Url, String),
    OpenAiApi(String, String),
}

impl From<OpenAiDelegateBuilderArgument> for (Client<OpenAIConfig>, String) {
    fn from(val: OpenAiDelegateBuilderArgument) -> Self {
        let (openai_config, model_name) = match val {
            OpenAiDelegateBuilderArgument::Endpoint(url, name) => {
                (OpenAIConfig::new().with_api_base(url), name)
            }
            OpenAiDelegateBuilderArgument::OpenAiApi(api_key, name) => {
                (OpenAIConfig::new().with_api_key(api_key), name)
            }
        };
        (Client::with_config(openai_config), model_name)
    }
}

pub(crate) struct OpenAiDelegateBuilder;

pub(crate) struct OpenAiDelegateBuilderWithEmbedding {
    embedding_client: Client<OpenAIConfig>,
    embedding_model_name: String,
}

pub(crate) struct OpenAiDelegateBuilderWithChat {
    chat_client: Client<OpenAIConfig>,
    chat_model_name: String,
}
pub(crate) struct OpenAiDelegateBuilderWithInstruct {
    instruct_client: Client<OpenAIConfig>,
    instruct_model_name: String,
}

impl OpenAiDelegateBuilderWithEmbedding {
    pub(crate) fn with_chat(self, endpoint: OpenAiDelegateBuilderArgument) -> OpenAiDelegate {
        let OpenAiDelegateBuilderWithEmbedding {
            embedding_client,
            embedding_model_name,
        } = self;
        let (chat_client, chat_model_name) = endpoint.into();
        OpenAiDelegate::new(
            LlmClient::Chat(ChatClient::new(chat_client, chat_model_name)),
            EmbeddingClient::new(embedding_client, embedding_model_name),
        )
    }
    pub(crate) fn with_instruct(self, endpoint: OpenAiDelegateBuilderArgument) -> OpenAiDelegate {
        let OpenAiDelegateBuilderWithEmbedding {
            embedding_client,
            embedding_model_name,
        } = self;
        let (instruct_client, instruct_model_name) = endpoint.into();
        OpenAiDelegate::new(
            LlmClient::Instruct(InstructClient::new(instruct_client, instruct_model_name)),
            EmbeddingClient::new(embedding_client, embedding_model_name),
        )
    }
}

impl OpenAiDelegateBuilderWithChat {
    pub(crate) fn with_embedding(self, endpoint: OpenAiDelegateBuilderArgument) -> OpenAiDelegate {
        let OpenAiDelegateBuilderWithChat {
            chat_client,
            chat_model_name,
        } = self;
        let (embedding_client, embedding_model_name) = endpoint.into();

        OpenAiDelegate::new(
            LlmClient::Chat(ChatClient::new(chat_client, chat_model_name)),
            EmbeddingClient::new(embedding_client, embedding_model_name),
        )
    }
}
impl OpenAiDelegateBuilderWithInstruct {
    pub(crate) fn with_embedding(self, endpoint: OpenAiDelegateBuilderArgument) -> OpenAiDelegate {
        let OpenAiDelegateBuilderWithInstruct {
            instruct_client,
            instruct_model_name,
        } = self;
        let (embedding_client, embedding_model_name) = endpoint.into();

        OpenAiDelegate::new(
            LlmClient::Instruct(InstructClient::new(instruct_client, instruct_model_name)),
            EmbeddingClient::new(embedding_client, embedding_model_name),
        )
    }
}

impl OpenAiDelegateBuilder {
    pub(crate) fn with_embedding(
        endpoint: OpenAiDelegateBuilderArgument,
    ) -> OpenAiDelegateBuilderWithEmbedding {
        let (embedding_client, embedding_model_name) = endpoint.into();

        OpenAiDelegateBuilderWithEmbedding {
            embedding_client,
            embedding_model_name,
        }
    }

    pub(crate) fn with_chat(
        endpoint: OpenAiDelegateBuilderArgument,
    ) -> OpenAiDelegateBuilderWithChat {
        let (chat_client, chat_model_name) = endpoint.into();
        OpenAiDelegateBuilderWithChat {
            chat_client,
            chat_model_name,
        }
    }
    pub(crate) fn with_instruct(
        endpoint: OpenAiDelegateBuilderArgument,
    ) -> OpenAiDelegateBuilderWithInstruct {
        let (instruct_client, instruct_model_name) = endpoint.into();

        OpenAiDelegateBuilderWithInstruct {
            instruct_client,
            instruct_model_name,
        }
    }
}
