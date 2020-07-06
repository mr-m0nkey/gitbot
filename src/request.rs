use rocket::{Data, Outcome};
use rocket::data;
use rocket::data::FromData;
use rocket::http::Status;
use rocket::request;
use rocket::request::Request;
use rocket::request::FromRequest;
use std;
use std::io::prelude::*;


const X_GITHUB_EVENT: &'static str = "X-GitHub-Event";

const PULL_REQUEST_EVENT: &'static str = "pull_request";
const ISSUE_COMMENT_EVENT: &'static str = "issue_comment";
const STATUS_EVENT: &'static str = "status";
const PUSH: &'static str = "push";

#[derive(Clone, Debug, PartialEq)]
pub enum GitHubEvent {
    PullRequest,
    IssueComment,
    Status,
    Push,
}

impl<'r, 'a> FromRequest<'r, 'a> for GitHubEvent {
    type Error = ();

    fn from_request(request: &'r Request<'a>) -> request::Outcome<GitHubEvent, ()> {
        let keys = request.headers().get(X_GITHUB_EVENT).collect::<Vec<_>>();
        if keys.len() != 1 {
            return Outcome::Failure((Status::BadRequest, ()));
        }

        let event = match keys[0] {
            PULL_REQUEST_EVENT => GitHubEvent::PullRequest,
            ISSUE_COMMENT_EVENT => GitHubEvent::IssueComment,
            STATUS_EVENT => GitHubEvent::Status,
            PUSH => GitHubEvent::Push,
            _ => { return Outcome::Failure((Status::BadRequest, ())); }
        };

        Outcome::Success(event)
    }
}