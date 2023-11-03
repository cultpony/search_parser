//#[cfg(test)]
//pub mod parser;
#[cfg(test)]
pub mod tokenizer;

use std::collections::HashSet;
use std::ops::Range;

use fake::faker::time::en::DateTimeBetween;
use fake::{Dummy, Fake};
use rand::prelude::Distribution;
use time::OffsetDateTime;

pub fn zero_time() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_546_300_800).unwrap()
}

pub fn max_time() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_696_860_500).unwrap()
}

#[derive(Debug, Dummy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
pub struct ImageMetadata {
    id: u64,
    #[dummy(faker = "DateTimeBetween(zero_time(), max_time())")]
    created_at: time::OffsetDateTime,
    #[dummy(faker = "32..80000")]
    height: u32,
    #[dummy(faker = "32..80000")]
    width: u32,
    #[dummy(faker = "2..100")]
    tags: Tags,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
pub struct Tags(HashSet<String>);

impl fake::Dummy<Range<u32>> for Tags {
    fn dummy_with_rng<R: fake::Rng + ?Sized>(config: &Range<u32>, rng: &mut R) -> Self {
        let count: u32 = rand::distributions::Uniform::new(config.start, config.end).sample(rng);
        let mut tags = HashSet::new();
        for _ in 0..count {
            tags.insert(fake::faker::lorem::en::Word().fake_with_rng(rng));
        }
        Self(tags)
    }
}

#[cfg(test)]
mod test {
    use std::hint::black_box;

    use super::ImageMetadata;
    use fake::{Fake, Faker};
    use rand::SeedableRng;

    pub fn es_authentication() -> Option<(String, String)> {
        None
    }

    pub fn es_request<S1: AsRef<str>, S2: AsRef<str>>(method: S1, path: S2) -> ureq::Request {
        let host =
            std::env::var("SPTEST_ES_HOST").unwrap_or_else(|_| "http://localhost:9200".to_string());
        let path = path.as_ref().trim_start_matches('/');
        let path = format!("{host}/{path}");
        match es_authentication() {
            None => ureq::agent().request(method.as_ref(), &path),
            Some(_) => todo!(),
        }
    }

    /// Create an ES index with random data (optionally with RNG seed)
    pub fn feed_elasticsearch<S: AsRef<str> + std::fmt::Display, T: Into<u64>>(
        index: S,
        seed: Option<T>,
    ) {
        let mut rng = rand_chacha::ChaChaRng::seed_from_u64(
            seed.map(|x| x.into()).unwrap_or(0x1199_2FFF_1BBA_9311),
        );
        let _images: Vec<ImageMetadata> = Vec::new();
        let res = es_request("PUT", format!("/{index}")).call().unwrap();
        assert!(res.status() < 400, "non-error code response");
        for _ in 0..1000000 {
            let image: ImageMetadata = Faker.fake_with_rng(&mut rng);
            let res = es_request("POST", format!("/{index}/_doc/"))
                .send_json(&image)
                .unwrap();
            assert!(res.status() < 400, "non-error code response");
        }

        let res = es_request("POST", format!("/{index}/_refresh"))
            .call()
            .unwrap();
        assert!(res.status() < 400, "non-error code response");
    }

    #[test]
    #[ignore = "only a test to see it works"]
    fn test_image_fakedata_gen() {
        let img: ImageMetadata = fake::Faker.fake();

        let img = black_box(img);
        feed_elasticsearch("test", Some(0x10u64));
        panic!("{img:?}");
    }
}
