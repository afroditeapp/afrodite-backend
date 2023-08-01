use super::{WriteCommandRunnerHandle, ResultSender, WriteCommandRunner, SendBack};






use error_stack::Result;




use crate::{
    api::{
        media::data::{HandleModerationRequest, Moderation, PrimaryImage},
        model::{
            AccountIdInternal, ContentId, ModerationRequestContent,
        },
    },
    server::data::{DatabaseError},
};

use super::{super::file::file::ImageSlot};



/// Synchronized write commands.
#[derive(Debug)]
pub enum MediaWriteCommand {
    SetModerationRequest {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        request: ModerationRequestContent,
    },
    GetModerationListAndCreateNewIfNecessary {
        s: ResultSender<Vec<Moderation>>,
        account_id: AccountIdInternal,
    },
    SaveToSlot {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    },
    UpdateModeration {
        s: ResultSender<()>,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    },
    UpdatePrimaryImage {
        s: ResultSender<()>,
        account_id: AccountIdInternal,
        primary_image: PrimaryImage,
    },
}


#[derive(Debug, Clone)]
pub struct MediaWriteCommandRunnerHandle<'a> {
    pub handle: &'a WriteCommandRunnerHandle,
}

impl MediaWriteCommandRunnerHandle<'_> {
    pub async fn set_moderation_request(
        &self,
        account_id: AccountIdInternal,
        request: ModerationRequestContent,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| MediaWriteCommand::SetModerationRequest {
            s,
            account_id,
            request,
        })
        .await
    }

    pub async fn get_moderation_list_and_create_if_necessary(
        &self,
        account_id: AccountIdInternal,
    ) -> Result<Vec<Moderation>, DatabaseError> {
        self.handle.send_event(|s| MediaWriteCommand::GetModerationListAndCreateNewIfNecessary {
            s,
            account_id,
        })
        .await
    }

    pub async fn update_moderation(
        &self,
        moderator_id: AccountIdInternal,
        moderation_request_owner: AccountIdInternal,
        result: HandleModerationRequest,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| MediaWriteCommand::UpdateModeration {
            s,
            moderator_id,
            moderation_request_owner,
            result,
        })
        .await
    }

    pub async fn update_primary_image(
        &self,
        account_id: AccountIdInternal,
        primary_image: PrimaryImage,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| MediaWriteCommand::UpdatePrimaryImage {
            s,
            account_id,
            primary_image,
        })
        .await
    }

    pub async fn save_to_slot(
        &self,
        account_id: AccountIdInternal,
        content_id: ContentId,
        slot: ImageSlot,
    ) -> Result<(), DatabaseError> {
        self.handle.send_event(|s| MediaWriteCommand::SaveToSlot {
            s,
            account_id,
            content_id,
            slot,
        })
        .await
    }
}


impl WriteCommandRunner {
    pub async fn handle_media_cmd(&self, cmd: MediaWriteCommand) {
        match cmd {
            MediaWriteCommand::SetModerationRequest {
                s,
                account_id,
                request,
            } => self
                .write()
                .set_moderation_request(account_id, request)
                .await
                .send(s),
            MediaWriteCommand::GetModerationListAndCreateNewIfNecessary { s, account_id } => self
                .write()
                .moderation_get_list_and_create_new_if_necessary(account_id)
                .await
                .send(s),
            MediaWriteCommand::SaveToSlot {
                s,
                account_id,
                content_id,
                slot,
            } => self
                .write()
                .save_to_slot(account_id, content_id, slot)
                .await
                .send(s),
            MediaWriteCommand::UpdateModeration {
                s,
                moderator_id,
                moderation_request_owner,
                result,
            } => self
                .write()
                .update_moderation(moderator_id, moderation_request_owner, result)
                .await
                .send(s),
            MediaWriteCommand::UpdatePrimaryImage {
                s,
                account_id,
                primary_image,
            } => self
                .write()
                .update_primary_image(account_id, primary_image)
                .await
                .send(s),
        }
    }
}
