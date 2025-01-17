use std::path::Path;

use rocket::{
    handler::Outcome,
    http::{uri::Segments, ContentType, Method, Status},
    outcome::IntoOutcome,
    response, Data, Handler, Request, Route,
};

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticFiles<const N: usize> {
    pub files: [File; N],
    pub rank: isize,
    pub root: &'static Path,
}

impl<const N: usize> StaticFiles<N> {
    pub fn rank(self, rank: isize) -> Self {
        StaticFiles { rank, ..self }
    }

    pub fn root(self, root: &'static Path) -> Self {
        StaticFiles { root, ..self }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct File {
    pub path: &'static Path,
    pub body: &'static [u8],
}

#[macro_export]
macro_rules! include_static {
    ($($path:literal),* $(,)?) => {
        crate::static_include::StaticFiles {
            files: [
                $(
                    crate::static_include::File {
                        path: std::path::Path::new($path),
                        body: include_bytes!(concat![env!("CARGO_MANIFEST_DIR"), "/", $path])
                    }
                ),*
            ],
            rank: 10,
            root: std::path::Path::new("static"),
        }
    }
}

impl<'r> response::Responder<'r> for File {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let mut response = self.body.respond_to(req)?;
        if let Some(ext) = self.path.extension() {
            if let Some(ct) = ContentType::from_extension(&ext.to_string_lossy()) {
                response.set_header(ct);
            }
        }
        Ok(response)
    }
}

impl<const N: usize> Handler for StaticFiles<N> {
    fn handle<'r>(&self, request: &'r Request, _data: Data) -> Outcome<'r> {
        let req_path = request
            .get_segments::<Segments>(0)
            .and_then(Result::ok)
            .and_then(|segments| segments.into_path_buf(false).ok())
            .into_outcome(Status::NotFound)?;

        Outcome::from(
            request,
            self.files
                .iter()
                .copied()
                .find(|File { path, .. }| req_path == path.file_name().unwrap()),
        )
    }
}

impl<const N: usize> From<StaticFiles<N>> for Vec<Route> {
    fn from(static_files: StaticFiles<N>) -> Vec<Route> {
        vec![Route::ranked(
            static_files.rank,
            Method::Get,
            "/<path..>",
            static_files,
        )]
    }
}
