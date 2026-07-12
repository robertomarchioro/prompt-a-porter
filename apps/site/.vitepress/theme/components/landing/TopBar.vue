<script setup lang="ts">
import { withBase } from 'vitepress'
import { GITHUB } from './links'
import { useLinkDownload } from './download'

const linkDownload = useLinkDownload()
</script>

<template>
  <header class="topbar">
    <div class="brand"><img class="mk" :src="withBase('/icons/icon-64.png')" alt="" width="30" height="30" /><span class="nm">Prompt&nbsp;a&nbsp;Porter</span></div>
    <nav class="nav">
      <a href="#collezione">La collezione</a>
      <a href="#come-funziona">Come funziona</a>
      <a :href="GITHUB">GitHub</a>
      <a class="dl" :href="linkDownload">Scarica</a>
    </nav>
  </header>
</template>
