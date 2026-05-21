type ApiError = {
  code: string;
  desc: string;
};

type ApiResponse<T> = {
  success: boolean;
  error: ApiError | null;
  data: T | null;
};

export async function apiMutator<T>(
  url: string,
  options?: RequestInit,
): Promise<T> {
  const response = await fetch(url, {
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      ...options?.headers,
    },
    ...options,
  });

  const payload = (await response.json()) as ApiResponse<T>;
  if (!response.ok || !payload.success) {
    throw new Error(payload.error?.desc ?? "Request failed");
  }

  return {
    data: payload,
    status: response.status,
    headers: response.headers,
  } as T;
}
