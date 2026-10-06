import { LoadingState, PUBLIC_CONTAINER } from "@/components/blog/public";
import { cn } from "@/lib/utils";

export default function BlogLoading() {
  return (
    <main className={cn(PUBLIC_CONTAINER, "grid gap-4 px-4 py-8")}>
      <LoadingState label="正在打开页面…" />
    </main>
  );
}
