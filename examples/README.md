# Examples

The examples are part of the framework surface. They are not throwaway demos.

Each one exists to pressure a different part of `ssw-rs`:

- `ssw-intake-demo`
  - Actix-first form flow
  - flash and CSRF hooks
  - field helpers, validation summary, and invalid redisplay
  - style-guide previews for badges, breadcrumbs, stat list, data table, and pagination
  - `/style-guide` for visual review of the current primitives
- `ssw-projects-demo`
  - larger page shell
  - list, detail, archive, and edit routes
  - top navigation, breadcrumbs, badges, stat list, pagination, empty states, metadata, validation summary, and 404 or 422 HTML responses
  - scoped CSS proof point in repeated card and badge UI
- `ssw-workers-demo`
  - Cloudflare Workers adapter proof
  - GET, POST, redirect, flash, CSRF, validation summary, fragment rendering, and HTML 404 flow
  - explicit asset route under the Workers runtime
  - keeps persistence and platform bindings such as D1 app-owned on purpose

## Running

### Intake demo

```bash
cargo run -p ssw-intake-demo
```

Open:

- `http://127.0.0.1:3000/`
- `http://127.0.0.1:3000/style-guide`

Route shape:

- `GET /`
  - empty intake form
- `POST /intake`
  - invalid submissions redisplay on the same page
  - valid submissions redirect to `/thanks`
- `GET /thanks`
  - success page with flash clearing
- `GET /style-guide`
  - visual review surface for current primitives
- `GET /assets/app.css`
  - explicit stylesheet route

### Projects demo

```bash
cargo run -p ssw-projects-demo
```

Open:

- `http://127.0.0.1:3002/projects`

Route shape:

- `GET /projects`
  - active projects list
- `GET /projects/archive`
  - empty-state route
- `GET /projects/{slug}`
  - detail page
- `GET /projects/{slug}/edit`
  - edit form
- `POST /projects/{slug}/edit`
  - invalid submissions return HTML `422`
  - missing projects return HTML `404`
- `GET /assets/app.css`
  - explicit stylesheet route

### Workers demo

```bash
cd examples/ssw-workers-demo
npm install
npm run dev
```

Open:

- `http://127.0.0.1:8788/`

Route shape:

- `GET /`
  - form page
- `POST /`
  - invalid submissions redisplay in place
  - valid submissions redirect to `/thanks`
- `GET /preview?note=...`
  - HTML fragment response
- `GET /thanks`
  - success page
- fallback
  - HTML `404` page through the shared response model
- `GET /assets/theme.css`
  - explicit stylesheet route

## Why this matters

The examples are where API pressure should show up first.

If a framework idea cannot stay clear in one of these apps, it probably should not be promoted into a public helper yet.
