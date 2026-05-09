<script lang="ts">
  import { PaneGroup, Pane, PaneResizer } from "paneforge";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import StatusBar from "$lib/components/StatusBar.svelte";
  // F1: shell strutturale. F2/F3/F4/F6 sostituiranno i placeholder
  // con i propri component (Sidebar, ListPane, DetailPane, RightRail).
</script>

<div class="shell-root">
  <TitleBar />
  <main class="shell-body">
    <PaneGroup direction="horizontal" autoSaveId="redesign-shell-v08">
      <Pane defaultSize={20} minSize={14} maxSize={30}>
        <div class="placeholder-pane sidebar-placeholder">
          <p>Sidebar (F2)</p>
        </div>
      </Pane>
      <PaneResizer class="resizer" />
      <Pane defaultSize={26} minSize={0} maxSize={40}>
        <div class="placeholder-pane list-placeholder">
          <p>List pane (F3)</p>
        </div>
      </Pane>
      <PaneResizer class="resizer" />
      <Pane defaultSize={54}>
        <div class="placeholder-pane detail-placeholder">
          <p>Detail / Editor (F4) + Right-rail (F6)</p>
        </div>
      </Pane>
    </PaneGroup>
  </main>
  <StatusBar />
</div>

<style>
  .shell-root {
    display: grid;
    grid-template-rows: var(--titlebar-h) 1fr var(--statusbar-h);
    height: 100vh;
    background: var(--bg-canvas);
    color: var(--text-default);
    font-family: var(--font-ui);
  }

  .shell-body {
    overflow: hidden;
  }

  .placeholder-pane {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    font-size: var(--fs-sm);
  }

  .sidebar-placeholder {
    background: var(--bg-surface);
  }

  .list-placeholder {
    background: var(--bg-canvas);
    border-left: 1px solid var(--border-subtle);
    border-right: 1px solid var(--border-subtle);
  }

  .detail-placeholder {
    background: var(--bg-canvas);
  }

  :global(.resizer) {
    width: 1px;
    background: var(--border-subtle);
    position: relative;
    cursor: col-resize;
    transition: background var(--motion-fast) var(--easing-standard);
  }

  :global(.resizer::after) {
    content: "";
    position: absolute;
    inset: 0 -3px;
  }

  :global(.resizer:hover),
  :global(.resizer[data-resize-handle-active]) {
    background: var(--accent-team);
  }
</style>
