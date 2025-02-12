use diesel::{prelude::{AsChangeset, Insertable, Queryable}, Selectable};
use model::{AccountId, ContentId, ReportProcessingState};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct MediaReport {
    pub processing_state: ReportProcessingState,
    pub content: MediaReportContent,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct UpdateMediaReport {
    pub target: AccountId,
    pub content: MediaReportContent,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct MediaReportContent {
    pub profile_content: Vec<ContentId>,
}

#[derive(Debug, Selectable, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = crate::schema::media_report)]
#[diesel(check_for_backend(crate::Db))]
pub struct MediaReportContentRaw {
    pub profile_content_uuid_0: Option<ContentId>,
    pub profile_content_uuid_1: Option<ContentId>,
    pub profile_content_uuid_2: Option<ContentId>,
    pub profile_content_uuid_3: Option<ContentId>,
    pub profile_content_uuid_4: Option<ContentId>,
    pub profile_content_uuid_5: Option<ContentId>,
}

impl MediaReportContentRaw {
    pub fn iter(&self) -> impl Iterator<Item=ContentId> {
        [
            self.profile_content_uuid_0,
            self.profile_content_uuid_1,
            self.profile_content_uuid_2,
            self.profile_content_uuid_3,
            self.profile_content_uuid_4,
            self.profile_content_uuid_5,
        ]
            .into_iter()
            .flatten()
    }
}
