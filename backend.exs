defmodule Glycerin.Backend do
  @moduledoc """
  Glycerin Browser Backend - Phase 11
  Manages network events, proxy rotation, and extension lifecycle
  """

  use GenServer
  require Logger

  # Client API

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  def handle_network_event(url, status, size) do
    GenServer.cast(__MODULE__, {:network_event, url, status, size})
  end

  def load_extension(script) do
    GenServer.cast(__MODULE__, {:load_extension, script})
  end

  def get_proxy_list do
    GenServer.call(__MODULE__, :get_proxy_list)
  end

  # Server Callbacks

  def init(_opts) do
    state = %{
      proxy_pool: generate_proxy_pool(10),
      proxy_index: 0,
      extensions: [],
      network_log: []
    }
    Logger.info("Glycerin Backend initialized")
    {:ok, state}
  end

  def handle_cast({:network_event, url, status, size}, state) do
    entry = %{
      timestamp: System.system_time(:second),
      url: url,
      status: status,
      size: size
    }

    log_entry = "[NET] #{url} -> #{status} (#{size} bytes)"
    Logger.info(log_entry)

    new_log = [entry | Enum.take(state.network_log, 99)]
    {:noreply, %{state | network_log: new_log}}
  end

  def handle_cast({:load_extension, script}, state) do
    ext = %{
      id: :erlang.unique_integer(),
      script: script,
      loaded_at: System.system_time(:second)
    }

    Logger.info("Extension loaded: #{ext.id}")
    {:noreply, %{state | extensions: [ext | state.extensions]}}
  end

  def handle_call(:get_proxy_list, _from, state) do
    {:reply, state.proxy_pool, state}
  end

  # Private Functions

  defp generate_proxy_pool(count) do
    # Generate simulated proxy list for DuckDuckGo rotation
    # In production: load from config or discover via P2P
    Enum.map(1..count, fn i ->
      "192.168.1.#{100 + i}:8080"
    end)
  end
end

# OTP Application Entry Point
defmodule Glycerin.App do
  use Application

  def start(_type, _args) do
    children = [
      Glycerin.Backend
    ]

    opts = [strategy: :one_for_one, name: Glycerin.Supervisor]
    Supervisor.start_link(children, opts)
  end
end
