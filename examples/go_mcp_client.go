// Package main provides a drop-in Go client for controlling the headless-engine via JSON-RPC 2.0.
package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os/exec"
)

type JsonRpcRequest struct {
	JsonRpc string      `json:"jsonrpc"`
	ID      int         `json:"id"`
	Method  string      `json:"method"`
	Params  interface{} `json:"params"`
}

type JsonRpcResponse struct {
	JsonRpc string          `json:"jsonrpc"`
	ID      *int            `json:"id,omitempty"`
	Result  json.RawMessage `json:"result,omitempty"`
	Error   *struct {
		Code    int    `json:"code"`
		Message string `json:"message"`
	} `json:"error,omitempty"`
}

type HeadlessClient struct {
	cmd    *exec.Cmd
	stdin  io.WriteCloser
	reader *bufio.Reader
	reqID  int
}

func NewHeadlessClient(binaryPath string) (*HeadlessClient, error) {
	cmd := exec.Command(binaryPath, "--stdio")
	stdin, err := cmd.StdinPipe()
	if err != nil {
		return nil, err
	}
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return nil, err
	}

	if err := cmd.Start(); err != nil {
		return nil, err
	}

	return &HeadlessClient{
		cmd:    cmd,
		stdin:  stdin,
		reader: bufio.NewReader(stdout),
		reqID:  1,
	}, nil
}

func (c *HeadlessClient) Call(method string, params interface{}) (string, error) {
	c.reqID++
	req := JsonRpcRequest{
		JsonRpc: "2.0",
		ID:      c.reqID,
		Method:  method,
		Params:  params,
	}

	data, err := json.Marshal(req)
	if err != nil {
		return "", err
	}

	if _, err := c.stdin.Write(append(data, '\n')); err != nil {
		return "", err
	}

	line, err := c.reader.ReadString('\n')
	if err != nil {
		return "", err
	}

	var resp JsonRpcResponse
	if err := json.Unmarshal([]byte(line), &resp); err != nil {
		return "", err
	}

	if resp.Error != nil {
		return "", fmt.Errorf("RPC Error [%d]: %s", resp.Error.Code, resp.Error.Message)
	}

	return string(resp.Result), nil
}

func (c *HeadlessClient) Close() error {
	c.Call("shutdown", map[string]string{})
	c.stdin.Close()
	return c.cmd.Wait()
}

func main() {
	fmt.Println(">>> Starting Headless Engine STDIO Client in Go...")
	client, err := NewHeadlessClient("./target/release/headless-engine.exe")
	if err != nil {
		// Fallback to debug binary if release not compiled yet
		client, err = NewHeadlessClient("./target/debug/headless-engine.exe")
		if err != nil {
			panic(fmt.Sprintf("Failed to start engine: %v", err))
		}
	}
	defer client.Close()

	// 1. Navigate
	fmt.Println("[1] Navigating to Wikipedia...")
	navRes, err := client.Call("tab.navigate", map[string]string{
		"url": "https://en.wikipedia.org/wiki/Quantum_computing",
	})
	if err != nil {
		panic(err)
	}
	fmt.Println("Navigation Report:", navRes)

	// 2. Extract Clean LLM Markdown
	fmt.Println("[2] Extracting Markdown for LLM...")
	mdRes, err := client.Call("tab.extractMarkdown", map[string]string{})
	if err != nil {
		panic(err)
	}
	fmt.Printf("Extracted Markdown Length: %d bytes\n", len(mdRes))

	// 3. Extract Links
	fmt.Println("[3] Extracting Actionable Links...")
	linksRes, err := client.Call("tab.extractLinks", map[string]string{})
	if err != nil {
		panic(err)
	}
	fmt.Println("Links Response Preview:", linksRes[:min(len(linksRes), 200)])
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
