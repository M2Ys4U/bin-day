# Binday Bot

Scrapes the Salford City Council website for the next bin collection day and posts the next collection day and which bins are scheduled for collection to a webhook URL.

Example systemd service and timer units are in the `systemd` directory.

## Building

To build, run `cargo build --release`

## Configuration

Set the following environment variables:

* `UPRN` - The [Unique Property Reference Number](https://en.wikipedia.org/wiki/Unique_Property_Reference_Number) for your property.

* `WEBHOOK_URL` - The URL of the webhook to post details of the next bin collections

* `OPERATOR_EMAIL` - Your email address, which is included in the user-agent header of the call to the SCC website.

* `WEBHOOK_SECRET` (optional) - The secret key used to generate the HMAC signature included in the `X-Signature-256` header sent with the webhook request.

## Example webhook payload

`{"date":"2023-08-16","black":false,"blue":false,"brown":true}`