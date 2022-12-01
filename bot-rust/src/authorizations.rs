use dashmap::DashMap;
use serenity::model::prelude::UserId as DiscordUserID;
use std::{future::Future, sync::Arc};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum Authorization {
    Allow,
    Deny,
}

pub type AuthorizationSender = oneshot::Sender<Authorization>;

#[derive(Debug, Clone)]
pub struct Authorizations {
    authorizations: Arc<DashMap<DiscordUserID, Option<AuthorizationSender>>>,
}

impl Authorizations {
    pub fn new() -> Self {
        Self {
            authorizations: Default::default(),
        }
    }

    fn send(&self, discord_user_id: DiscordUserID, authorization: Authorization) -> bool {
        if let Some((_, Some(sender))) = self.authorizations.remove(&discord_user_id) {
            sender.send(authorization).unwrap();
            return true;
        }
        return false;
    }

    pub fn allow(&self, discord_user_id: DiscordUserID) -> bool {
        self.send(discord_user_id, Authorization::Allow)
    }

    pub fn deny(&self, discord_user_id: DiscordUserID) -> bool {
        self.send(discord_user_id, Authorization::Deny)
    }

    pub fn request_authorization(
        &self,
        discord_user_id: DiscordUserID,
    ) -> impl Future<Output = Option<Authorization>> + '_ {
        let (sender, receiver) = oneshot::channel();
        self.authorizations.insert(discord_user_id, Some(sender));
        async move {
            let authorization = receiver.await.ok();
            if authorization.is_some() {
                self.authorizations.remove(&discord_user_id);
            }
            authorization
        }
    }
}
