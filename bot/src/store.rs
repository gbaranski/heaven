use dashmap::DashMap;
use serenity::model::prelude::UserId as DiscordUserID;
use std::{sync::Arc, future::Future};
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum Authorization {
    Allow,
    Deny,
}

pub type AuthorizationSender = oneshot::Sender<Authorization>;

#[derive(Debug, Clone)]
pub struct Store {
    authorizations: Arc<DashMap<DiscordUserID, Option<AuthorizationSender>>>,
}

impl Store {
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
    
    pub fn get_authorization(&self, discord_user_id: DiscordUserID) -> impl Future<Output = Authorization> + '_ {
        let (sender, receiver) = oneshot::channel();
        self.authorizations.insert(discord_user_id, Some(sender));
        async move {
            let authorization = receiver.await.unwrap();
            self.authorizations.remove(&discord_user_id);
            authorization
        }
    }
}
