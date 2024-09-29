use wiremock::{
    matchers::{
        method,
        path,
    },
    Mock,
    ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    let response = app
        .post_subscriptions(body.into())
        .await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into())
        .await;

    // Assert
    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_returns_a_200_with_valid_email_confirmation_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let body = "name=de%20Pprog&email=pprog%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response = app
        .post_subscriptions(body.into())
        .await;

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "pprog@gmail.com");
    assert_eq!(saved.name, "de Pprog");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_name_email_is_invalid() {
    // Arrange
    let app = spawn_app().await;
    let long_name = "A".repeat(257);

    let long_name_email = format!("name={}&email=%40gmail.com", long_name);

    let test_cases = vec![
        (
            "name=&email=ursula_le_guin%40gmail.com".to_owned(),
            "empty name".to_owned(),
        ),
        (long_name_email, "name too long".to_owned()),
        (
            "name=U(o){&email=definitely-not-an-email".to_owned(),
            "name has forbidden characters".to_owned(),
        ),
        ("name=ursula&email=".to_owned(), "empty email".to_owned()),
        (
            "name=ursula&email=asdf".to_owned(),
            "wrong formatted email".to_owned(),
        ),
    ];
    for (body, description) in test_cases {
        // Act
        let response = app
            .post_subscriptions(body.into())
            .await;
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 BAD REQUEST when the payload was {}.",
            description
        );
    }
}
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app
            .post_subscriptions(invalid_body.into())
            .await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=pprog%20rammingg&email=pprog_lez_go%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into())
        .await;

    // Assert
    // Mock asserts on drop
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into())
        .await;

    // Assert
    // Get the first intercepted request
    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()[0];

    let confirmation_links = app.get_confirmation_links(email_request);

    // The two links should be identical
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    // Sabotage the database
    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
        .execute(&app.db_pool)
        .await
        .unwrap();
    // Act
    let response = app
        .post_subscriptions(body.into())
        .await;
    // Assert
    assert_eq!(response.status().as_u16(), 500);
}
