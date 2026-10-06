import { NextResponse } from "next/server";

export const runtime = "nodejs";

function getInternalApiBaseUrl() {
  const base =
    process.env.INTERNAL_API_URL || process.env.NEXT_PUBLIC_API_URL || "http://backend:8080/api/v1";
  return base.trim().replace(/\/+$/, "");
}

export async function GET(_request: Request, { params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  if (!/^\d+$/.test(id)) {
    return NextResponse.json({ code: 400, message: "无效的文件编号", data: null }, { status: 400 });
  }

  try {
    const response = await fetch(`${getInternalApiBaseUrl()}/files/${id}`, {
      cache: "force-cache",
      signal: AbortSignal.timeout(15_000),
    });
    if (!response.ok) {
      return NextResponse.json(
        { code: response.status, message: "图片不存在或暂时无法读取", data: null },
        { status: response.status },
      );
    }

    const headers = new Headers();
    for (const name of ["content-type", "content-length", "etag", "last-modified"]) {
      const value = response.headers.get(name);
      if (value) headers.set(name, value);
    }
    headers.set("Cache-Control", "public, max-age=31536000, immutable");

    return new Response(response.body, { status: 200, headers });
  } catch (error) {
    console.error("File proxy failed:", error);
    return NextResponse.json(
      { code: 502, message: "图片服务暂时不可用", data: null },
      { status: 502 },
    );
  }
}
