use crate::{ap_url, instance::Instance, schema::tags, Connection, Error, Result};
use diesel::{self, ExpressionMethods, QueryDsl, RunQueryDsl};
use plume_common::activity_pub::Hashtag;

#[derive(Clone, Identifiable, Queryable)]
pub struct Tag {
    pub id: i32,
    pub tag: String,
    pub is_hashtag: bool,
    pub post_id: i32,
}

#[derive(Insertable)]
#[table_name = "tags"]
pub struct NewTag {
    pub tag: String,
    pub is_hashtag: bool,
    pub post_id: i32,
}

impl Tag {
    insert!(tags, NewTag);
    get!(tags);
    find_by!(tags, find_by_name, tag as &str);
    list_by!(tags, for_post, post_id as i32);

    pub fn to_activity(&self) -> Result<Hashtag> {
        let mut ht = Hashtag::default();
        ht.set_href_string(ap_url(&format!(
            "{}/tag/{}",
            Instance::get_local()?.public_domain,
            self.tag
        )))?;
        ht.set_name_string(self.tag.clone())?;
        Ok(ht)
    }

    pub fn from_activity(
        conn: &Connection,
        tag: &Hashtag,
        post: i32,
        is_hashtag: bool,
    ) -> Result<Tag> {
        Tag::insert(
            conn,
            NewTag {
                tag: tag.name_string()?,
                is_hashtag,
                post_id: post,
            },
        )
    }

    pub fn build_activity(tag: String) -> Result<Hashtag> {
        let mut ht = Hashtag::default();
        ht.set_href_string(ap_url(&format!(
            "{}/tag/{}",
            Instance::get_local()?.public_domain,
            tag
        )))?;
        ht.set_name_string(tag)?;
        Ok(ht)
    }

    pub fn delete(&self, conn: &Connection) -> Result<()> {
        diesel::delete(self)
            .execute(conn)
            .map(|_| ())
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::db;
    use crate::{diesel::Connection, instance::tests::fill_database};
    use assert_json_diff::assert_json_eq;
    use serde_json::to_value;

    #[test]
    fn to_activity() {
        let conn = &db();
        conn.test_transaction::<_, Error, _>(|| {
            fill_database(conn);
            let tag = Tag {
                id: 0,
                tag: "a_tag".into(),
                is_hashtag: false,
                post_id: 0,
            };
            let act = tag.to_activity()?;
            let expected = json!({
                "href": "https://plu.me/tag/a_tag",
                "name": "a_tag",
                "type": "Hashtag"
            });

            assert_json_eq!(to_value(&act)?, expected);

            Ok(())
        })
    }

    #[test]
    fn build_activity() {
        let conn = &db();
        conn.test_transaction::<_, Error, _>(|| {
            fill_database(conn);
            let act = Tag::build_activity("a_tag".into())?;
            let expected = json!({
                "href": "https://plu.me/tag/a_tag",
                "name": "a_tag",
                "type": "Hashtag"
            });

            assert_json_eq!(to_value(&act)?, expected);

            Ok(())
        });
    }
}
