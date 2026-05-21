export async function apiMutator(url, options) {
    const response = await fetch(url, {
        credentials: "include",
        headers: {
            "Content-Type": "application/json",
            ...options?.headers,
        },
        ...options,
    });
    const payload = (await response.json());
    if (!response.ok || !payload.success) {
        throw new Error(payload.error?.desc ?? "Request failed");
    }
    if (payload.data === null) {
        throw new Error("Response did not include data");
    }
    return payload.data;
}
