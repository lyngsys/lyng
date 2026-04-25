# Lyng site

The public documentation and blog at <https://lyngsys.github.io/lyng/>.
Built with [Astro](https://astro.build/) and
[Starlight](https://starlight.astro.build/) and deployed to GitHub Pages
by `.github/workflows/pages.yml` on every push to `main` that touches
`site/**`.

## Local development

```sh
cd site
npm install
npm run dev      # serves http://localhost:4321/lyng/
npm run build    # writes the static site to site/dist/
npm run preview  # serves the built site
```

## Layout

```
site/
├── astro.config.mjs           # site, base, Starlight config
├── src/
│   ├── content.config.ts      # docs + blog collection schemas
│   ├── content/
│   │   ├── docs/              # Starlight-rendered docs
│   │   └── blog/              # blog posts (one .md per post)
│   ├── pages/
│   │   ├── blog/index.astro   # blog index
│   │   ├── blog/[...slug].astro # per-post page
│   │   └── rss.xml.js         # RSS feed
│   └── styles/blog.css        # shared blog styles
└── public/                    # static assets (favicon, images)
```

## Adding a docs page

Drop a Markdown or MDX file under `src/content/docs/`. The path becomes
the URL. To make it appear in the sidebar, add an entry to the `sidebar`
array in `astro.config.mjs`.

## Adding a blog post

Create `src/content/blog/YYYY-MM-DD-slug.md` with this frontmatter:

```yaml
---
title: "Post title"
description: "One-sentence summary used in listings and RSS."
pubDate: 2026-04-25
author: "Your name"          # optional, defaults to "The Lyng team"
draft: false                  # set true to hide from listings + RSS
---
```

The post's URL is `/blog/<filename-without-extension>/`.

## Internal vs. public docs

The Markdown under the workspace `docs/` directory and each crate's
`docs/` directory is **internal engineering documentation** and is not
published. Only content under `site/src/content/` is rendered to the
public site.
