use super::ObjectMetadata;
use crate::storage::{Error, ErrorKind, Fileinfo};
use chrono::prelude::*;
use serde::Deserialize;
use std::{iter::Extend, path::PathBuf};

#[derive(Deserialize, Debug)]
pub(crate) struct ResponseBody {
    items: Option<Vec<Item>>,
    prefixes: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Item {
    name: String,
    updated: DateTime<Utc>,
    size: String,
}

impl ResponseBody {
    pub(crate) fn list(self) -> Result<Vec<Fileinfo<PathBuf, ObjectMetadata>>, Error> {
        let files: Vec<Fileinfo<PathBuf, ObjectMetadata>> = self.items.map_or(Ok(vec![]), move |items: Vec<Item>| {
            items
                .iter()
                .filter(|item: &&Item| !item.name.ends_with('/'))
                .map(move |item: &Item| item.to_file_info())
                .collect()
        })?;
        let dirs: Vec<Fileinfo<PathBuf, ObjectMetadata>> = self.prefixes.map_or(Ok(vec![]), |prefixes: Vec<String>| {
            prefixes
                .iter()
                .filter(|prefix| *prefix != "//")
                .map(|prefix| prefix_to_file_info(prefix))
                .collect()
        })?;
        let result: &mut Vec<Fileinfo<PathBuf, ObjectMetadata>> = &mut vec![];
        result.extend(dirs);
        result.extend(files);
        Ok(result.to_vec())
    }
}

impl Item {
    pub(crate) fn to_metadata(&self) -> Result<ObjectMetadata, Error> {
        let size: u64 = self.size.parse().map_err(|e| Error::new(ErrorKind::TransientFileNotAvailable, e))?;

        Ok(ObjectMetadata {
            size,
            last_updated: Some(self.updated.into()),
            is_file: !self.name.ends_with('/'),
        })
    }

    pub(crate) fn to_file_info(&self) -> Result<Fileinfo<PathBuf, ObjectMetadata>, Error> {
        let path: PathBuf = PathBuf::from(self.name.clone());
        let metadata: ObjectMetadata = self.to_metadata()?;

        Ok(Fileinfo { metadata, path })
    }
}

pub(crate) fn prefix_to_file_info(prefix: &str) -> Result<Fileinfo<PathBuf, ObjectMetadata>, Error> {
    Ok(Fileinfo {
        path: prefix.into(),
        metadata: ObjectMetadata {
            last_updated: None,
            is_file: false,
            size: 0,
        },
    })
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::storage::storage_backend::Metadata;
    use std::time::SystemTime;

    #[test]
    fn to_metadata() {
        let sys_time: SystemTime = SystemTime::now();
        let date_time: DateTime<Utc> = DateTime::from(sys_time);

        let item: Item = Item {
            name: "".into(),
            updated: date_time,
            size: "50".into(),
        };

        let metadata: ObjectMetadata = item.to_metadata().unwrap();
        assert_eq!(metadata.size, 50);
        assert_eq!(metadata.modified().unwrap(), sys_time);
        assert_eq!(metadata.is_file, true);
    }

    #[test]
    fn to_metadata_parse_error() {
        use chrono::prelude::Utc;

        let item: Item = Item {
            name: "".into(),
            updated: Utc::now(),
            size: "unparseable".into(),
        };

        let metadata: Result<ObjectMetadata, Error> = item.to_metadata();
        assert_eq!(metadata.err().unwrap().kind(), ErrorKind::TransientFileNotAvailable);
    }
}
