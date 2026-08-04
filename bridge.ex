defmodule Glycerin.Bridge do
  @moduledoc """
  FFI Bridge between Rust Engine and Elixir Backend
  Uses Port protocol for zero-copy communication
  """

  use GenServer
  require Logger

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  def init(_opts) do
    # Spawn Rust engine as port
    port = Port.open({:spawn_executable, './target/release/glycerin'}, [
      {:args, []},
      :binary,
      {:packet, 2}
    ])

    Logger.info("Rust engine port spawned")
    {:ok, %{port: port, buffer: <<>>}}
  end

  def handle_info({port, {:data, data}}, %{port: port} = state) do
    # Parse FlatBuffer message from Rust
    message = parse_flatbuffer(data)
    Logger.info("Received from Rust: #{inspect(message)}")

    # Dispatch to backend
    case message do
      %{type: "NetworkResponse", url: url, status: status, size: size} ->
        Glycerin.Backend.handle_network_event(url, status, size)

      _ ->
        Logger.debug("Unknown message type")
    end

    {:noreply, state}
  end

  def handle_info({port, {:exit_status, code}}, %{port: port} = state) do
    Logger.error("Rust engine exited with code: #{code}")
    {:stop, :normal, state}
  end

  defp parse_flatbuffer(data) do
    # Simplified FlatBuffer parsing
    # In production: use generated Elixir code from protocol.fbs
    %{type: "Unknown", raw: data}
  end
end
