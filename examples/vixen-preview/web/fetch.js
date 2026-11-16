// Limit interception to this page's private API prefix; assets still use the network.
export function createFetch(handleRequest, baseUrl, networkFetch = globalThis.fetch.bind(globalThis)) {
  const scope = new URL('/__axum__/', baseUrl);

  return async (input, init) => {
    const url = new URL(input instanceof Request ? input.url : input, baseUrl);
    if (url.origin !== scope.origin || !url.pathname.startsWith(scope.pathname)) {
      return networkFetch(input, init);
    }

    const request = new Request(input instanceof Request ? input : url, init);
    request.signal.throwIfAborted();
    const path = `/${url.pathname.slice(scope.pathname.length)}${url.search}`;
    const response = await handleRequest(path, request);
    request.signal.throwIfAborted();
    return response;
  };
}
