import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Zero to Hero: Rust',
  description: 'A self-paced, project-based 30-day curriculum — from absolute beginner to production-minded Rust developer.',

  srcDir: '.',
  outDir: '.vitepress/dist',
  cacheDir: '.vitepress/cache',

  srcExclude: [
    'node_modules/**',
    '.vitepress/**',
    '**/target/**',
    'CLAUDE.md',
  ],

  ignoreDeadLinks: true,

  head: [
    ['link', { rel: 'icon', href: 'https://www.rust-lang.org/static/images/rust-logo-blk.svg' }],
  ],

  themeConfig: {
    logo: 'https://www.rust-lang.org/static/images/rust-logo-blk.svg',

    nav: [
      { text: 'Curriculum', link: '/CURRICULUM' },
      { text: 'Primitives', link: '/PRIMITIVES' },
      { text: 'Data Structures', link: '/DATA_STRUCTURES' },
      {
        text: 'GitHub',
        link: 'https://github.com/mmussett/zero2hero-rust',
      },
    ],

    sidebar: [
      {
        text: 'Getting Started',
        items: [
          { text: 'Curriculum Overview', link: '/CURRICULUM' },
        ],
      },
      {
        text: 'Week 1 — Foundations',
        collapsed: false,
        items: [
          { text: 'Day 01: Toolchain, Cargo, Hello World', link: '/day-01/README' },
          { text: 'Day 02: Variables, Types, Control Flow', link: '/day-02/README' },
          { text: 'Day 03: Functions and Structs', link: '/day-03/README' },
          { text: 'Day 04: Ownership and Move Semantics', link: '/day-04/README' },
          { text: 'Day 05: Borrowing, References, Lifetimes', link: '/day-05/README' },
          { text: 'Day 06: Enums and Pattern Matching', link: '/day-06/README' },
          { text: 'Day 07: Modules and Packages', link: '/day-07/README' },
        ],
      },
      {
        text: 'Week 2 — Core Language',
        collapsed: false,
        items: [
          { text: 'Day 08: Collections', link: '/day-08/README' },
          { text: 'Day 09: Option and Result', link: '/day-09/README' },
          { text: 'Day 10: Traits', link: '/day-10/README' },
          { text: 'Day 11: Generics and Data Structures', link: '/day-11/README' },
          { text: 'Day 12: Lifetimes (Mastery)', link: '/day-12/README' },
          { text: 'Day 13: Closures and Iterators', link: '/day-13/README' },
          { text: 'Day 14: Testing', link: '/day-14/README' },
        ],
      },
      {
        text: 'Week 3 — Intermediate Patterns',
        collapsed: false,
        items: [
          { text: 'Day 15: Trait Objects', link: '/day-15/README' },
          { text: 'Day 16: Smart Pointers', link: '/day-16/README' },
          { text: 'Day 17: Concurrency', link: '/day-17/README' },
          { text: 'Day 18: Async and Tokio', link: '/day-18/README' },
          { text: 'Day 19: File I/O and Serialization', link: '/day-19/README' },
          { text: 'Day 20: CLI Applications', link: '/day-20/README' },
          { text: 'Day 21: WebAssembly', link: '/day-21/README' },
        ],
      },
      {
        text: 'Week 4 — Production',
        collapsed: false,
        items: [
          { text: 'Day 22: HTTP Server (axum)', link: '/day-22/README' },
          { text: 'Day 23: Database (sqlx)', link: '/day-23/README' },
          { text: 'Day 24: Error Handling at Scale', link: '/day-24/README' },
          { text: 'Day 25: Logging and Observability', link: '/day-25/README' },
          { text: 'Day 26: Benchmarking', link: '/day-26/README' },
          { text: 'Day 27: Unsafe Rust', link: '/day-27/README' },
          { text: 'Day 28: Macros', link: '/day-28/README' },
          { text: 'Day 29: Publishing', link: '/day-29/README' },
          { text: 'Day 30: Capstone', link: '/day-30/README' },
        ],
      },
      {
        text: 'Reference',
        collapsed: true,
        items: [
          { text: 'Primitive Types', link: '/PRIMITIVES' },
          { text: 'Data Structures', link: '/DATA_STRUCTURES' },
        ],
      },
    ],

    search: {
      provider: 'local',
    },

    editLink: {
      pattern: 'https://github.com/mmussett/zero2hero-rust/edit/main/:path',
      text: 'Edit this page on GitHub',
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/mmussett/zero2hero-rust' },
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Zero to Hero: Rust',
    },

    lastUpdated: {
      text: 'Last updated',
    },
  },

  markdown: {
    theme: {
      light: 'github-light',
      dark:  'one-dark-pro',
    },
    lineNumbers: true,
  },
})
