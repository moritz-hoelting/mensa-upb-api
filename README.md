# Mensa UPB API

Web scraper for the canteen of the University of Paderborn.

## Configuration

The following environment variables are available:

| Variable                 | Description                                                      | Default            |
| ------------------------ | ---------------------------------------------------------------- | ------------------ |
| `API_INTERFACE`          | The interface the API should listen on.                          | `127.0.0.1`        |
| `API_PORT`               | The port the API should listen on.                               | `8080`             |
| `API_CORS_ALLOWED`       | The allowed origins for CORS requests.                           | None, set manually |
| `API_RATE_LIMIT_SECONDS` | The time in seconds after which the rate limit should replenish. | `5`                |
| `API_RATE_LIMIT_BURST`   | The maximum number of requests that can be made in a burst.      | `5`                |
