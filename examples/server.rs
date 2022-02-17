use rocket::response::content::Html;
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::time::{self, Duration};
use rocket::{get, launch, routes};
use std::time::SystemTime;

#[get("/events")]
fn events() -> EventStream![] {
    let mut id = 0;
    let mut interval = time::interval(Duration::from_secs(2));
    EventStream! {
        loop {
            interval.tick().await;
            let unix_time = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            id += 1;
            yield Event::data(unix_time.to_string()).id(id.to_string());
        }
    }
}

#[get("/")]
fn index() -> Html<&'static str> {
    Html(
        r#"
Open Console
<script>
    const es = new EventSource("http://localhost:8000/events");
    es.onopen = () => console.log("Connection Open!");
    es.onmessage = (e) => console.log("Message:", e);
    es.onerror = (e) => {
        console.log("Error:", e);
        es.close();
    };
</script>
"#,
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![events, index])
}
