import { NextResponse } from "next/server";

export const runtime = "nodejs";

function getInternalApiBaseUrl() {
  const base =
    process.env.INTERNAL_API_URL || process.env.NEXT_PUBLIC_API_URL || "http://backend:8080/api/v1";
  return base.trim().replace(/\/+$/, "");
}

export async function POST(request: Request) {
  const contentType = request.headers.get("content-type") || "";
  if (!contentType.toLowerCase().startsWith("multipart/form-data")) {
    return NextResponse.json(
      { code: 400, message: "请求必须包含图片文件", data: null },
      { status: 400 },
    );
  }

  const headers = new Headers({ "Content-Type": contentType });
  const authorization = request.headers.get("authorization");
  if (authorization) headers.set("Authorization", authorization);

  try {
    const response = await fetch(`${getInternalApiBaseUrl()}/admin/files/upload`, {
      method: "POST",
      headers,
      body: await request.arrayBuffer(),
      cache: "no-store",
      signal: AbortSignal.timeout(60_000),
    });

    return new Response(await response.arrayBuffer(), {
      status: response.status,
      headers: {
        "Content-Type": response.headers.get("content-type") || "application/json",
      },
    });
  } catch (error) {
    console.error("Upload proxy failed:", error);
    return NextResponse.json(
      { code: 502, message: "上传服务暂时不可用，请稍后重试", data: null },
      { status: 502 },
    );
  }
}
