---
title: Deployment
description: Deploy your sukr site to any static hosting platform
weight: 1
---

sukr builds your site to `public/`. This directory contains self-contained static HTML, CSS, and assets — no server-side runtime needed. Upload it anywhere that serves static files. If you haven't built a site yet, start with the [Getting Started](getting-started.html) guide.

## Local Preview

Preview your site locally before deploying:

```bash
cd public
python3 -m http.server 8000
```

Open `http://localhost:8000` in your browser.

## GitHub Pages

1. Push your repository to GitHub
2. Build your site: `sukr`
3. Deploy the `public/` directory using one of:
   - **GitHub Actions** — add a workflow that runs `sukr` and deploys `public/` to Pages
   - **Manual** — push the `public/` contents to a `gh-pages` branch

Example workflow (`.github/workflows/deploy.yml`):

```yaml
name: Deploy
on:
  push:
    branches: [main]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install --path .
      - run: sukr
      - uses: peaceiris/actions-gh-pages@v4
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./public
```

## Netlify

1. Connect your repository in the Netlify dashboard
2. Set build command: `cargo install --path . && sukr`
3. Set publish directory: `public`

Netlify detects changes and rebuilds automatically on push.

## Cloudflare Pages

1. Connect your repository in the Cloudflare Pages dashboard
2. Set build command: `cargo install --path . && sukr`
3. Set build output directory: `public`

## Any Static Host

For any host that serves static files (S3, DigitalOcean Spaces, a VPS):

```bash
sukr
rsync -avz public/ user@server:/var/www/mysite/
```

Or use `scp`, `aws s3 sync`, or your host's CLI tool.

## Security Headers

sukr outputs zero JavaScript, which means you can set a strict Content Security Policy that blocks all script execution. See [Security](security.html) for recommended CSP headers and platform-specific configuration.
