<template>
  <div class="filo-wrap">
    <svg viewBox="0 0 340 34" class="filo" aria-hidden="true">
      <path
        d="M4 24 C 40 6, 80 6, 116 24 S 192 42, 228 24 S 300 6, 330 20"
        fill="none"
        stroke="#c89a55"
        stroke-width="1.6"
        stroke-dasharray="7 6"
        stroke-linecap="round"
      />
      <g transform="translate(330 20) rotate(24)">
        <path d="M0 0 L 20 -2 L 20 2 Z" fill="#b9b6ac" />
        <line x1="20" y1="0" x2="30" y2="0" stroke="#c7c4ba" stroke-width="2.4" stroke-linecap="round" />
        <circle cx="4" cy="0" r="2.4" fill="none" stroke="#8c8a80" stroke-width="1" />
      </g>
    </svg>
  </div>
</template>

<style scoped>
.filo-wrap {
  padding: 10px 20px 4px;
}
.filo {
  width: 100%;
  height: 30px;
  overflow: visible;
}
</style>
