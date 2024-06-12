use okapi::openapi3::{RefOr, Response as OpenApiReponse, Responses};
use rocket_okapi::OpenApiError;
use schemars::Map;

type ResponseDescription = (&'static str, &'static str); // (status_code, description)

pub fn get_openapi_responses(
    descriptions: Vec<ResponseDescription>,
) -> Result<Responses, OpenApiError> {
    let mut responses = Map::new();

    for (status_code, description) in descriptions {
        responses.insert(
            status_code.to_string(),
            RefOr::Object(OpenApiReponse {
                description: description.to_string(),
                ..Default::default()
            }),
        );
    }

    Ok(Responses {
        responses,
        ..Default::default()
    })
}
