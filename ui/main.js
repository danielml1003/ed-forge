const tauri = window.__TAURI__;

if (!tauri?.core?.invoke || !tauri?.event?.listen) {
  const root = document.querySelector(".shell");
  if (root) {
    const warning = document.createElement("section");
    warning.className = "card";
    warning.innerHTML = "<h2>Runtime Error</h2><p class='muted'>Tauri API is unavailable. Start with <code>npm run dev</code> inside the app folder.</p>";
    root.prepend(warning);
  }
  throw new Error("Tauri API unavailable");
}

const { invoke } = tauri.core;
const { listen } = tauri.event;

const tabs = document.querySelectorAll(".tab");
const screens = document.querySelectorAll(".screen");

const discoverForm = document.querySelector("#discover-filter-form");
const discoverQueryEl = document.querySelector("#discover-query");
const discoverProviderEl = document.querySelector("#discover-provider");
const discoverListEl = document.querySelector("#discover-list");
const discoverDetailEl = document.querySelector("#discover-detail");
const discoverRefreshButton = document.querySelector("#discover-refresh");
const activityLogEl = document.querySelector("#activity-log");

const libraryListEl = document.querySelector("#library-list");
const librarySummaryEl = document.querySelector("#library-summary");
const libraryRefreshButton = document.querySelector("#library-refresh");

const runtimeForm = document.querySelector("#runtime-form");
const runtimeLowResourceEl = document.querySelector("#runtime-low-resource");
const runtimeIngestionEl = document.querySelector("#runtime-ingestion");
const runtimeSyncEl = document.querySelector("#runtime-sync");
const runtimeOverviewEl = document.querySelector("#runtime-overview");

const state = {
  providers: [],
  apps: [],
  selectedAppId: null,
  library: [],
};

function activateScreen(targetId) {
  tabs.forEach((tab) => tab.classList.toggle("active", tab.dataset.target === targetId));
  screens.forEach((screen) => screen.classList.toggle("active", screen.id === targetId));
}

function logActivity(line) {
  if (!activityLogEl) return;
  const el = document.createElement("div");
  el.className = "log-item";
  el.textContent = `${new Date().toLocaleTimeString()} ${line}`;
  activityLogEl.prepend(el);
}

function renderProviders() {
  if (!discoverProviderEl) return;
  discoverProviderEl.innerHTML = '<option value="">All Providers</option>';
  for (const provider of state.providers) {
    const option = document.createElement("option");
    option.value = provider.id;
    option.textContent = `${provider.name} (${provider.region})`;
    discoverProviderEl.append(option);
  }
}

function renderDiscoverDetail() {
  if (!discoverDetailEl) return;
  const app = state.apps.find((entry) => entry.id === state.selectedAppId);
  if (!app) {
    discoverDetailEl.textContent = "Pick an app to inspect details.";
    return;
  }

  discoverDetailEl.innerHTML = `
    <div><strong>${app.name}</strong></div>
    <div class="muted">Provider: ${app.providerName}</div>
    <div class="muted">Category: ${app.category}</div>
    <div class="muted">Price: $${app.priceUsd.toFixed(2)}</div>
    <div class="muted">Rating: ${app.rating.toFixed(1)} / 5</div>
    <div class="muted">Stock: ${app.stock}</div>
    <div class="muted">Source: ${app.sourceUrl}</div>
  `;
}

function renderDiscoverList() {
  if (!discoverListEl) return;
  if (!state.apps.length) {
    discoverListEl.innerHTML = '<div class="muted">No apps found for this filter.</div>';
    return;
  }

  discoverListEl.innerHTML = "";
  for (const app of state.apps) {
    const card = document.createElement("article");
    card.className = "item";
    card.innerHTML = `
      <div class="item-title">${app.name}</div>
      <div class="item-meta">${app.providerName} • ${app.category} • $${app.priceUsd.toFixed(2)}</div>
      <div class="item-actions">
        <button type="button" data-action="detail" data-id="${app.id}">Details</button>
        <button type="button" data-action="save" data-id="${app.id}">Save to Library</button>
      </div>
    `;
    discoverListEl.append(card);
  }
}

function renderLibrary() {
  if (!libraryListEl || !librarySummaryEl) return;
  if (!state.library.length) {
    librarySummaryEl.textContent = "No apps saved.";
    libraryListEl.innerHTML = '<div class="muted">Save apps from Discover and they appear here.</div>';
    return;
  }

  const runningCount = state.library.filter((app) => app.state === "running").length;
  librarySummaryEl.textContent = `${state.library.length} apps saved, ${runningCount} running.`;

  libraryListEl.innerHTML = "";
  for (const app of state.library) {
    const card = document.createElement("article");
    card.className = "item";
    card.innerHTML = `
      <div class="item-title">${app.name}</div>
      <div class="item-meta">${app.providerName} • ${app.category} • v${app.version} • ${app.state}</div>
      <div class="item-meta">Last launched: ${app.lastLaunched || "never"}</div>
      <div class="item-actions">
        <button type="button" data-action="launch" data-id="${app.id}">Launch</button>
        <button type="button" data-action="remove" data-id="${app.id}">Remove</button>
      </div>
    `;
    libraryListEl.append(card);
  }
}

function renderRuntime(config) {
  if (!runtimeOverviewEl || !runtimeLowResourceEl || !runtimeIngestionEl || !runtimeSyncEl) return;

  runtimeLowResourceEl.checked = Boolean(config.lowResourceMode);
  runtimeIngestionEl.checked = Boolean(config.ingestionEnabled);
  runtimeSyncEl.value = String(config.syncIntervalSec);

  runtimeOverviewEl.innerHTML = `
    <div class="muted">Low resource mode: ${config.lowResourceMode ? "on" : "off"}</div>
    <div class="muted">Data ingestion: ${config.ingestionEnabled ? "enabled" : "disabled"}</div>
    <div class="muted">Sync interval: ${config.syncIntervalSec}s</div>
    <div class="muted">Managed apps: ${config.libraryCount}</div>
    <div class="muted">Running apps: ${config.runningCount}</div>
  `;
}

async function loadDiscover() {
  const query = (discoverQueryEl?.value ?? "").trim();
  const provider = (discoverProviderEl?.value ?? "").trim();

  state.apps = await invoke("store_list_items", {
    query: query || null,
    provider: provider || null,
  });

  renderDiscoverList();
  renderDiscoverDetail();
}

async function loadLibrary() {
  state.library = await invoke("library_list_apps");
  renderLibrary();
}

async function loadRuntime() {
  const config = await invoke("runtime_get_config");
  renderRuntime(config);
}

async function initialize() {
  state.providers = await invoke("store_list_providers");
  renderProviders();
  await Promise.all([loadDiscover(), loadLibrary(), loadRuntime()]);
}

tabs.forEach((tab) => {
  tab.addEventListener("click", () => {
    const targetId = tab.dataset.target;
    if (!targetId) return;
    activateScreen(targetId);
  });
});

discoverForm?.addEventListener("submit", async (event) => {
  event.preventDefault();
  await loadDiscover();
});

discoverRefreshButton?.addEventListener("click", async () => {
  const result = await invoke("store_refresh_cache");
  logActivity(`sources refreshed: ${result.items} apps from ${result.providers} providers`);
  await loadDiscover();
});

discoverListEl?.addEventListener("click", async (event) => {
  const button = event.target.closest("button[data-action]");
  if (!button) return;

  const action = button.dataset.action;
  const id = button.dataset.id;
  if (!id) return;

  if (action === "detail") {
    const app = await invoke("store_get_item", { item: id });
    if (!app) return;
    state.selectedAppId = id;
    renderDiscoverDetail();
    return;
  }

  if (action === "save") {
    const saved = await invoke("library_save_item", { item: id });
    if (saved) {
      logActivity(`saved to library: ${saved.name}`);
      await Promise.all([loadLibrary(), loadRuntime()]);
    }
  }
});

libraryListEl?.addEventListener("click", async (event) => {
  const button = event.target.closest("button[data-action]");
  if (!button) return;

  const action = button.dataset.action;
  const id = button.dataset.id;
  if (!id) return;

  if (action === "launch") {
    const launched = await invoke("library_launch_item", { item: id });
    if (launched) {
      logActivity(`launched app: ${launched.name}`);
      await Promise.all([loadLibrary(), loadRuntime()]);
    }
    return;
  }

  if (action === "remove") {
    const removed = await invoke("library_remove_item", { item: id });
    if (removed) {
      logActivity(`removed from library: ${id}`);
      await Promise.all([loadLibrary(), loadRuntime()]);
    }
  }
});

libraryRefreshButton?.addEventListener("click", async () => {
  await Promise.all([loadLibrary(), loadRuntime()]);
  logActivity("library refreshed");
});

runtimeForm?.addEventListener("submit", async (event) => {
  event.preventDefault();

  const payload = {
    lowResourceMode: Boolean(runtimeLowResourceEl?.checked),
    ingestionEnabled: Boolean(runtimeIngestionEl?.checked),
    syncIntervalSec: Number(runtimeSyncEl?.value || 30),
  };

  await invoke("runtime_update_config", { payload });
  logActivity("runtime settings updated");
  await loadRuntime();
});

(async () => {
  await listen("store-refreshed", () => {
    void loadDiscover();
  });

  await listen("library-updated", async () => {
    await Promise.all([loadLibrary(), loadRuntime()]);
  });

  await initialize();
})();
