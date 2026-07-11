<script setup lang="ts">
import { GITHUB, RELEASES } from './links'
</script>

<template>
  <header class="topbar">
    <div class="brand"><span class="mk serif">P</span><span class="nm">Prompt&nbsp;a&nbsp;Porter</span></div>
    <nav class="nav">
      <a href="#collezione">La collezione</a>
      <a href="#come-funziona">Come funziona</a>
      <a :href="GITHUB">GitHub</a>
      <a class="dl" :href="RELEASES">Scarica</a>
    </nav>
  </header>
</template>
