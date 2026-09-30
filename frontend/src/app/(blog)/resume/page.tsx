import type { Metadata } from "next";

import { ResumeViewer } from "@/components/resume/resume-viewer";
import { resumeApi } from "@/lib/api";
import type { Resume } from "@/types";
import { absoluteUrl } from "@/lib/seo";

export const dynamic = "force-dynamic";

export const metadata: Metadata = {
  title: "个人简历",
  description: "个人经历、技术方向与项目经验，正在寻找工作机会，欢迎查看与联系。",
  alternates: { canonical: absoluteUrl("/resume") },
};

export default async function ResumePage() {
  let resume: Resume | null = null;

  try {
    const payload = await resumeApi.getPublic();
    resume = payload.resume;
  } catch (error) {
    console.error("Failed to fetch resume:", error);
  }

  return <ResumeViewer resume={resume} />;
}
