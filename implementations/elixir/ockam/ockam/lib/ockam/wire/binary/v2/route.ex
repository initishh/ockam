defmodule Ockam.Wire.Binary.V2.Route do
  @moduledoc false

  alias Ockam.Wire.Binary.V2.Address
  alias Ockam.Wire.DecodeError
  alias Ockam.Wire.EncodeError

  require DecodeError
  require EncodeError

  def encode(route) when is_list(route) do
    case encode_addresses(route, []) do
      {:error, error} ->
        {:error, error}

      encoded_addresses ->
        {:ok, encoded_addresses}
    end
  end

  def encode(input) do
    reason = {:argument_is_not_a_route, input}
    {:error, EncodeError.new(reason)}
  end

  def encode_addresses([], encoded), do: Enum.reverse(encoded)

  def encode_addresses([address | remaining_route], encoded) do
    case Address.encode(address) do
      {:error, error} -> {:error, error}
      encoded_address -> encode_addresses(remaining_route, [encoded_address | encoded])
    end
  end

  @spec decode(maybe_improper_list) :: list
  @doc """
  Decodes a route from a binary.

  Returns {:ok, routes} if it succeeds.
  Returns {:error, successful_routes} if it fails.
  """
  def decode(addresses) when is_list(addresses) and length(addresses) > 0 do
    # TODO: this is also kinda ugly
    decoded = Enum.map(addresses, fn(address) ->
      Address.decode(address)
    end)
    if length(decoded) == length(addresses) do
      {:ok, decoded}
    else
      # should return an actual error instead of only successful routes.
      {:error, decoded}
    end
  end

  def decode([]), do: {:ok, []}
end