import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import sitemap from '@astrojs/sitemap';

// https://astro.build/config
export default defineConfig({
  site: 'https://lyngsys.github.io',
  base: '/lyng',
  trailingSlash: 'ignore',
  integrations: [
    starlight({
      title: 'Lyng',
      description:
        'Browser and runtime infrastructure: a JavaScript engine and an HTML parser, written in Rust.',
      social: {
        github: 'https://github.com/lyngsys/lyng',
      },
      sidebar: [
        {
          label: 'Lyng JS',
          items: [
            { label: 'Overview', slug: 'lyng-js/overview' },
            { label: 'Getting started', slug: 'lyng-js/getting-started' },
          ],
        },
        {
          label: 'HTML Parser',
          items: [{ label: 'Overview', slug: 'html-parser/overview' }],
        },
        {
          label: 'Blog',
          link: '/blog/',
        },
      ],
      customCss: ['./src/styles/blog.css'],
    }),
    sitemap(),
  ],
});
