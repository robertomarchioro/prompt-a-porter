<script lang="ts">
  interface Props {
    larghezza?: string;
    altezza?: string;
  }

  let { larghezza = "100%", altezza = "16px" }: Props = $props();
</script>

<div class="skel" style:width={larghezza} style:height={altezza}></div>

<style>
  .skel {
    background: linear-gradient(
      90deg,
      var(--bg-overlay) 0%,
      var(--bg-raised) 50%,
      var(--bg-overlay) 100%
    );
    background-size: 200% 100%;
    animation: skel 1.4s linear infinite;
    border-radius: var(--radius-sm);
  }

  @keyframes skel {
    from {
      background-position: 200% 0;
    }
    to {
      background-position: -200% 0;
    }
  }
</style>
