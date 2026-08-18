export declare const VERSION = "1.0.0";
export declare const REPO = "yutuknown/headless-engine";
export interface NavigationReport {
    status: number;
    requested_url: string;
    final_url: string;
    page_title: string;
    is_captcha_detected: boolean;
    html_bytes: number;
}
export interface LinkInfo {
    text: string;
    href: string;
}
export interface FormInfo {
    action: string;
    method: string;
    inputs: Array<{
        name: string;
        input_type: string;
        value: string;
        placeholder: string;
    }>;
}
export interface SearchResults {
    page_title: string;
    ai_overview?: {
        summary: string;
        source_references: string[];
    };
    knowledge_panel?: {
        title: string;
        subtitle: string;
        description: string;
        attributes: Array<[string, string]>;
    };
    video_results: Array<{
        title: string;
        video_id: string;
        url: string;
        channel: string;
        duration: string;
    }>;
    news_results: Array<{
        headline: string;
        source: string;
        time_ago: string;
        link: string;
    }>;
    image_results: Array<{
        title: string;
        image_url: string;
        source_url: string;
        domain: string;
    }>;
    organic_results: Array<{
        title: string;
        link: string;
        snippet: string;
    }>;
    total_results_found: number;
}
export declare function resolveBinary(explicitPath?: string): string;
export declare class HeadlessBrowser {
    private process;
    private rl;
    private reqId;
    private pendingCallbacks;
    constructor(binaryPath?: string);
    private call;
    navigate(url: string, tabId?: string): Promise<NavigationReport>;
    extractMarkdown(selector?: string, tabId?: string): Promise<string>;
    extractResults(tabId?: string): Promise<SearchResults>;
    extractLinks(tabId?: string): Promise<LinkInfo[]>;
    extractForms(tabId?: string): Promise<FormInfo[]>;
    click(target: string, tabId?: string): Promise<any>;
    typeText(selector: string, text: string, tabId?: string): Promise<string>;
    evaluateJs(code: string, tabId?: string): Promise<string>;
    createTab(profile?: string): Promise<string>;
    closeTab(tabId: string): Promise<boolean>;
    close(): void;
}
