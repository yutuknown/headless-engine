// Package headless provides a pure Go client to control the Headless Engine via JSON-RPC 2.0.
package headless

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"sync"
)

type Client struct {
	cmd    *exec.Cmd
	stdin  io.WriteCloser
	reader *bufio.Reader
	reqID  int
	mu     sync.Mutex
}

type rpcRequest struct {
	JsonRpc string      `json:"jsonrpc"`
	ID      int         `json:"id"`
	Method  string      `json:"method"`
	Params  interface{} `json:"params"`
}

type rpcResponse struct {
	JsonRpc string          `json:"jsonrpc"`
	ID      *int            `json:"id,omitempty"`
	Result  json.RawMessage `json:"result,omitempty"`
	Error   *struct {
		Code    int    `json:"code"`
		Message string `json:"message"`
	} `json:"error,omitempty"`
}

type NavigationReport struct {
	Status            int    `json:"status"`
	RequestedUrl      string `json:"requested_url"`
	FinalUrl          string `json:"final_url"`
	PageTitle         string `json:"page_title"`
	IsCaptchaDetected bool   `json:"is_captcha_detected"`
	HtmlBytes         int    `json:"html_bytes"`
}

func NewClient(binaryPath string) (*Client, error) {
	if binaryPath == "" {
		binaryPath = findBinary()
	}

	cmd := exec.Command(binaryPath, "--stdio")
	stdin, err := cmd.StdinPipe()
	if err != nil {
		return nil, fmt.Errorf("failed to open stdin: %w", err)
	}

	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return nil, fmt.Errorf("failed to open stdout: %w", err)
	}

	if err := cmd.Start(); err != nil {
		return nil, fmt.Errorf("failed to start headless engine: %w", err)
	}

	return &Client{
		cmd:    cmd,
		stdin:  stdin,
		reader: bufio.NewReader(stdout),
		reqID:  1,
	}, nil
}

func findBinary() string {
	candidates := []string{
		"headless-engine",
		"headless-engine.exe",
		"./target/release/headless-engine",
		"./target/release/headless-engine.exe",
		"./target/debug/headless-engine",
		"./target/debug/headless-engine.exe",
	}
	for _, c := range candidates {
		if _, err := os.Stat(c); err == nil {
			return c
		}
	}
	return "headless-engine"
}

func (c *Client) Call(method string, params interface{}) (json.RawMessage, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	c.reqID++
	req := rpcRequest{
		JsonRpc: "2.0",
		ID:      c.reqID,
		Method:  method,
		Params:  params,
	}

	data, err := json.Marshal(req)
	if err != nil {
		return nil, err
	}

	if _, err := c.stdin.Write(append(data, '\n')); err != nil {
		return nil, err
	}

	line, err := c.reader.ReadString('\n')
	if err != nil {
		return nil, err
	}

	var resp rpcResponse
	if err := json.Unmarshal([]byte(line), &resp); err != nil {
		return nil, err
	}

	if resp.Error != nil {
		return nil, fmt.Errorf("rpc error [%d]: %s", resp.Error.Code, resp.Error.Message)
	}

	return resp.Result, nil
}

func (c *Client) Navigate(url string) (*NavigationReport, error) {
	raw, err := c.Call("tab.navigate", map[string]string{"url": url})
	if err != nil {
		return nil, err
	}
	var report NavigationReport
	if err := json.Unmarshal(raw, &report); err != nil {
		return nil, err
	}
	return &report, nil
}

func (c *Client) ExtractMarkdown(selector string) (string, error) {
	params := map[string]string{}
	if selector != "" {
		params["selector"] = selector
	}
	raw, err := c.Call("tab.extractMarkdown", params)
	if err != nil {
		return "", err
	}
	var res struct {
		Markdown string `json:"markdown"`
	}
	if err := json.Unmarshal(raw, &res); err != nil {
		return "", err
	}
	return res.Markdown, nil
}

func (c *Client) Close() error {
	c.Call("shutdown", map[string]string{})
	c.stdin.Close()
	return c.cmd.Wait()
}
