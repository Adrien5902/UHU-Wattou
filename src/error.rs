use crate::group::GroupId;
use serenity::all::GuildId;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum WattouError {
    #[error("Cette commande ne peut pas être utilisée en mp")]
    CommandCanOnlyBeUsedInGuilds,
    #[error("Le groupe {1} n'exsite pas pour le serveur {0}")]
    NoGroupForGuild(GuildId, GroupId),
    #[error("Aucune données trouvées pour le serveur {0}")]
    NoDataForGuild(GuildId),
    #[error("Échec du parsing du message")]
    MessageParsingFailed,
    #[error("Groupe introuvable")]
    GroupNotFound,
    #[error("Colle parsing failed {0}")]
    ColleParsingFailed(ColleParsingError),
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum ColleParsingError {
    #[error("Id parsing failed")]
    IdParsingFailed,
    #[error("Unknown")]
    Unknown,
}
