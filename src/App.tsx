import { useEffect, useState } from "react";
import {
  deleteInstance,
  getWslVersion,
  installDistribution,
  listInstances,
  listOnlineDistributions,
  logMessage,
  openTerminal,
  renameInstance,
  setDefaultInstance,
  shutdown,
  startInstance,
  stopInstance,
} from "./api";
import type { WslInstance, WslVersion } from "./types";
import "./App.css";

function stateClass(state: string) {
  switch (state) {
    case "Running":
      return "state-running";
    case "Stopped":
      return "state-stopped";
    default:
      return "state-unknown";
  }
}

function App() {
  const [instances, setInstances] = useState<WslInstance[]>([]);
  const [selectedName, setSelectedName] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [working, setWorking] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [wslVersion, setWslVersion] = useState<WslVersion | null>(null);

  const [createOpen, setCreateOpen] = useState(false);
  const [onlineDistros, setOnlineDistros] = useState<string[]>([]);
  const [selectedDistro, setSelectedDistro] = useState<string>("");
  const [createInstallName, setCreateInstallName] = useState<string>("");

  const [renameOpen, setRenameOpen] = useState(false);
  const [newName, setNewName] = useState<string>("");

  const selected =
    instances.find((i) => i.name === selectedName) || instances[0];

  const showMessage = (msg: string) => {
    setMessage(msg);
    setTimeout(() => setMessage(null), 4000);
  };

  const refresh = async () => {
    setRefreshing(true);
    try {
      const list = await listInstances();
      setInstances(list);
      if (!selectedName && list.length > 0) {
        setSelectedName(list[0].name);
      }
    } catch (err) {
      showMessage(`Failed to load instances: ${err}`);
    } finally {
      setRefreshing(false);
    }
  };

  useEffect(() => {
    logMessage("App component mounted");
    refresh();
    getWslVersion()
      .then(setWslVersion)
      .catch(() => {
        // Keep the default unavailable state; do not spam the user with toast.
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const withWorking = async (
    action: () => Promise<void>,
    successMsg: string,
    shouldRefresh = true
  ) => {
    try {
      setWorking(true);
      await action();
      showMessage(successMsg);
      if (shouldRefresh) {
        await refresh();
      }
    } catch (err) {
      showMessage(`Error: ${err}`);
    } finally {
      setWorking(false);
    }
  };

  const handleStart = () => {
    if (!selected) return;
    withWorking(
      () => startInstance(selected.name),
      `Started ${selected.name}`
    );
  };

  const handleStop = () => {
    if (!selected) return;
    withWorking(
      () => stopInstance(selected.name),
      `Stopped ${selected.name}`
    );
  };

  const handleShutdown = () => {
    if (!window.confirm("Shutdown all running WSL instances and the WSL 2 VM?")) {
      return;
    }
    withWorking(shutdown, "WSL shutdown complete");
  };

  const handleOpenTerminal = () => {
    if (!selected) return;
    openTerminal(selected.name).catch((err) => showMessage(`Error: ${err}`));
  };

  const handleSetDefault = () => {
    if (!selected) return;
    withWorking(
      () => setDefaultInstance(selected.name),
      `Set ${selected.name} as default`
    );
  };

  const handleDelete = () => {
    if (!selected) return;
    if (
      !window.confirm(
        `Are you sure you want to delete "${selected.name}"?\n\nThis action cannot be undone and all data in this instance will be lost.`
      )
    ) {
      return;
    }
    withWorking(
      () => deleteInstance(selected.name),
      `Deleted ${selected.name}`
    );
  };

  const openCreateDialog = async () => {
    setCreateOpen(true);
    setWorking(true);
    try {
      const distros = await listOnlineDistributions();
      setOnlineDistros(distros);
      if (distros.length > 0) {
        setSelectedDistro(distros[0]);
      }
    } catch (err) {
      showMessage(`Failed to load online distributions: ${err}`);
      setCreateOpen(false);
    } finally {
      setWorking(false);
    }
  };

  const handleCreate = () => {
    if (!selectedDistro) return;
    withWorking(
      () => installDistribution(selectedDistro, createInstallName.trim()),
      `Installing ${createInstallName.trim() || selectedDistro}. WSL is downloading in the background; the instance will appear after it completes.`,
      false
    );
    setCreateOpen(false);
    setCreateInstallName("");
    // Auto-refresh a few times to catch the new instance once WSL finishes.
    setTimeout(refresh, 3000);
    setTimeout(refresh, 8000);
    setTimeout(refresh, 15000);
  };

  const openRenameDialog = () => {
    if (!selected) return;
    setNewName(selected.name);
    setRenameOpen(true);
  };

  const handleRename = () => {
    if (!selected || !newName.trim() || newName.trim() === selected.name) {
      showMessage("Please enter a different name");
      return;
    }
    if (
      !window.confirm(
        `Rename "${selected.name}" to "${newName.trim()}"?\n\nThis exports the instance, imports it under the new name, and removes the old registration. The data is preserved but the instance becomes an imported distribution.`
      )
    ) {
      return;
    }
    withWorking(
      () => renameInstance(selected.name, newName.trim()),
      `Renamed ${selected.name} to ${newName.trim()}`
    );
    setRenameOpen(false);
    setNewName("");
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>WSL Manager</h1>
        <div className="header-actions">
          <span className="wsl-version" title={wslVersion?.raw || ""}>
            WSL {wslVersion?.wslVersion || "unavailable"}
          </span>
          <button onClick={openCreateDialog} disabled={working || refreshing}>
            Create Instance
          </button>
          <button onClick={handleShutdown} disabled={working || refreshing}>
            Shutdown All
          </button>
          <button onClick={refresh} disabled={refreshing}>
            {refreshing ? "Refreshing..." : "Refresh"}
          </button>
        </div>
      </header>

      {message && <div className="toast">{message}</div>}

      {createOpen && (
        <div className="modal-overlay" onClick={() => setCreateOpen(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>Create New WSL Instance</h3>
            <p className="modal-hint">
              This will run{" "}
              <code>wsl --install -d &lt;distro&gt; --name &lt;name&gt; --no-launch</code>.
              It may require administrator permission and download the distribution from Microsoft Store.
            </p>
            <label htmlFor="distro-select">Distribution</label>
            <select
              id="distro-select"
              value={selectedDistro}
              onChange={(e) => setSelectedDistro(e.target.value)}
            >
              {onlineDistros.length === 0 ? (
                <option value="">Loading...</option>
              ) : (
                onlineDistros.map((distro) => (
                  <option key={distro} value={distro}>
                    {distro}
                  </option>
                ))
              )}
            </select>
            <label htmlFor="install-name">Instance Name (optional)</label>
            <input
              id="install-name"
              type="text"
              value={createInstallName}
              onChange={(e) => setCreateInstallName(e.target.value)}
              placeholder={selectedDistro}
            />
            <p className="modal-hint">
              Leave empty to use the default name. Use a unique name if you want multiple copies of the same distribution.
            </p>
            <div className="modal-actions">
              <button onClick={() => setCreateOpen(false)}>Cancel</button>
              <button
                className="primary"
                onClick={handleCreate}
                disabled={!selectedDistro || working}
              >
                Install
              </button>
            </div>
          </div>
        </div>
      )}

      {renameOpen && selected && (
        <div className="modal-overlay" onClick={() => setRenameOpen(false)}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h3>Rename Instance</h3>
            <p className="modal-hint">
              Rename <strong>{selected.name}</strong> to a new name. This preserves all data but converts the instance to an imported distribution.
            </p>
            <label htmlFor="new-name">New Name</label>
            <input
              id="new-name"
              type="text"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="new-name"
            />
            <div className="modal-actions">
              <button onClick={() => setRenameOpen(false)}>Cancel</button>
              <button
                className="primary"
                onClick={handleRename}
                disabled={!newName.trim() || working}
              >
                Rename
              </button>
            </div>
          </div>
        </div>
      )}

      <main className="app-body">
        <aside className="instance-list">
          <h2>Instances</h2>
          {instances.length === 0 ? (
            <p className="empty">No WSL instances found.</p>
          ) : (
            <ul>
              {instances.map((instance) => (
                <li
                  key={instance.name}
                  className={instance.name === selectedName ? "active" : ""}
                  onClick={() => setSelectedName(instance.name)}
                >
                  <div className="instance-row">
                    <span className="instance-name">
                      {instance.default && <span className="default-badge">★</span>}
                      {instance.name}
                    </span>
                    <span className={`instance-state ${stateClass(instance.state)}`}>
                      {instance.state}
                    </span>
                  </div>
                  <div className="instance-meta">
                    {instance.distribution || instance.name} · WSL {instance.version}
                  </div>
                </li>
              ))}
            </ul>
          )}
        </aside>

        <section className="instance-detail">
          {selected ? (
            <>
              <div className="detail-header">
                <h2>
                  {selected.name}
                  {selected.default && (
                    <span className="default-label">Default</span>
                  )}
                </h2>
                <span className={`state-pill ${stateClass(selected.state)}`}>
                  {selected.state}
                </span>
              </div>

              <div className="detail-grid">
                <div className="detail-item">
                  <span className="label">Distribution</span>
                  <span className="value">{selected.distribution || "-"}</span>
                </div>
                <div className="detail-item">
                  <span className="label">WSL Version</span>
                  <span className="value">{selected.version}</span>
                </div>
                <div
                  className="detail-item"
                  title="All WSL 2 instances share the same VM IP"
                >
                  <span className="label">WSL2 VM IP</span>
                  <span className="value">
                    {selected.state === "Running"
                      ? selected.ipAddress || "-"
                      : "-"}
                  </span>
                </div>
                <div className="detail-item">
                  <span className="label">Default Instance</span>
                  <span className="value">
                    {selected.default ? "Yes" : "No"}
                  </span>
                </div>
              </div>

              <div className="action-groups">
                <div className="action-group">
                  <h3>Lifecycle</h3>
                  <div className="buttons">
                    <button
                      onClick={handleStart}
                      disabled={working || selected.state === "Running"}
                    >
                      Start
                    </button>
                    <button
                      onClick={handleStop}
                      disabled={working || selected.state !== "Running"}
                    >
                      Stop
                    </button>
                    <button onClick={handleOpenTerminal} disabled={working}>
                      Open Terminal
                    </button>
                  </div>
                </div>

                <div className="action-group">
                  <h3>Configuration</h3>
                  <div className="buttons">
                    <button
                      onClick={handleSetDefault}
                      disabled={working || selected.default}
                    >
                      Set as Default
                    </button>
                    <button onClick={openRenameDialog} disabled={working}>
                      Rename
                    </button>
                  </div>
                </div>

                <div className="action-group danger">
                  <h3>Danger Zone</h3>
                  <div className="buttons">
                    <button
                      className="danger"
                      onClick={handleDelete}
                      disabled={working}
                    >
                      Delete Instance
                    </button>
                  </div>
                </div>
              </div>
            </>
          ) : (
            <p className="empty">Select an instance to view details.</p>
          )}
        </section>
      </main>
    </div>
  );
}

export default App;
