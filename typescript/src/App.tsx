import React, { useEffect, useRef, useState } from "react";
import {
  CustomEventDataType,
  CustomEventType,
  SSE,
  SSEOptions,
  SSEOptionsMethod,
} from "sse-ts";
import "./App.css";

interface Message {
  User?: string;
  Assistant?: [string, [string, string][]];
}
interface PartialMessage {
  message_content?: string;
  source?: [string, string];
}
interface Conversation extends Array<Message> {}

function App() {
  const [inputText, setInputText] = useState<string>("");
  const [conversation, setConversation] = useState<Conversation>([]);
  const [partialMessage, setPartialMessage] = useState<PartialMessage | null>(
    null
  );
  const messagesEndRef = useRef<null | HTMLDivElement>(null);
  const [tooltip, setTooltip] = useState<{
    visible: boolean;
    x: number;
    y: number;
    text: string;
  }>({ visible: false, x: 0, y: 0, text: "" });

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [conversation]);

  function handleMouseEvents(
    event: React.MouseEvent<HTMLElement>,
    text: string
  ) {
    const element = event.currentTarget.getBoundingClientRect();
    setTooltip({
      visible: !tooltip.visible,
      x: element.x,
      y: element.y,
      text: text,
    });
  }

  async function submit() {
    const userMessageArray = [{ User: inputText }];

    // First we add the message from the user
    setConversation((prev) => [...prev, ...userMessageArray]);
    setInputText("");

    const sseOptions: SSEOptions = {
      headers: { "Content-Type": "application/json", "api-key": "apiKey" },
      method: SSEOptionsMethod.POST,
      payload: JSON.stringify([...conversation, ...userMessageArray]),
    };

    // Open a connection to the SSE endpoint
    const source = new SSE(
      "https://oracle-rs.semanticallyinvalid.net/streaming_conversation",
      sseOptions
    );

    // Handle the message events from the server
    source.addEventListener("message", (event: CustomEventType) => {
      const dataEvent = event as CustomEventDataType;
      const data: PartialMessage = JSON.parse(dataEvent.data);
      setPartialMessage(data);

      if (data.message_content || data.source) {
        setConversation((prev) => {
          if (data.message_content) {
            // If message_content is present, we need to update the last message with additional content
            const lastMessage = prev[prev.length - 1];
            if (lastMessage.Assistant) {
              lastMessage.Assistant[0] += data.message_content;
            } else {
              lastMessage.Assistant = [data.message_content, []];
            }
            return [...prev.slice(0, prev.length - 1), lastMessage];
          } else if (data.source) {
            // If source is present, we need to add a new link to the last message
            const lastMessage = prev[prev.length - 1];
            if (lastMessage.Assistant) {
              lastMessage.Assistant[1].push(data.source);
            } else {
              lastMessage.Assistant = ["", [data.source]];
            }
            return [...prev.slice(0, prev.length - 1), lastMessage];
          } else {
            return [...prev.slice(0, prev.length - 1)];
          }
        });
      }
    });
  }

  return (
    <div className="App">
      <header className="App-header">
        <div className="message-list">
          {conversation.map((message, idx) => {
            if (message.User) {
              return (
                <div key={idx} className="user-text">
                  <p>{message.User}</p>
                </div>
              );
            }
            if (message.Assistant) {
              return (
                <div key={idx} className="assistant-text">
                  <p>{message.Assistant[0]}</p>
                  <ul
                    style={{
                      display: "flex",
                      flexDirection: "row",
                      gap: "8px",
                    }}
                  >
                    {message.Assistant[1].map((url, urlIdx) => (
                      <li key={urlIdx} style={{ listStyleType: "none" }}>
                        <div
                          className="link-bubble"
                          onClick={(e) => handleMouseEvents(e, url[1])}
                        >
                          {url[0]}
                        </div>
                      </li>
                    ))}
                  </ul>
                  {tooltip.visible && (
                    <div
                      className="tooltip-text"
                      // style={{
                      //   position: "absolute",
                      //   top: tooltip.y + 100,
                      //   left: tooltip.x,
                      // }}
                    >
                      {tooltip.text}
                    </div>
                  )}
                </div>
              );
            }
            return null; // this is just to handle cases where neither User nor Assistant properties are present, though it shouldn't occur based on your data structure
          })}
          <div ref={messagesEndRef} />
        </div>

        <div
          style={{
            position: "fixed",
            bottom: 0,
            display: "flex",
            width: "100%",
            padding: 16,
            boxSizing: "border-box",
            backgroundColor: "#282c34",
          }}
        >
          <input
            type="text"
            onChange={(e) => setInputText(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                submit();
              }
            }}
            value={inputText}
            style={{
              width: "80%",
              marginRight: 8,
              padding: 8,
              borderRadius: 4,
              border: "none",
            }}
          />
          <button
            onClick={submit}
            style={{
              width: "20%",
              backgroundColor: "#61dafb",
              border: "none",
              color: "#282c34",
              padding: 8,
              borderRadius: 4,
            }}
          >
            Submit
          </button>
        </div>
      </header>
    </div>
  );
}

export default App;
