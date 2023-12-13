import React, { useEffect, useReducer, useRef, useState } from "react";
import { SSE } from "sse.js";
import "./App.css";
import AssistantResponse from "./AssistantResponse";
interface Source {
  ordinal: number;
  index: number;
  citation: string;
  url: string;
  origin_text: string;
}

interface Message {
  User?: string;
  Assistant?: [string, Source[]];
}

interface PartialAssistant {
  content?: string;
  source?: Source;
  finished?: string;
}

interface Conversation extends Array<Message> {}

interface AddMessageAction {
  type: "ADD_MESSAGE";
  payload: Message;
}

interface UpdateAssistantMessageAction {
  type: "UPDATE_ASSISTANT_MESSAGE";
  payload: {
    content: string;
  };
}
interface UpdateAssistantSourceAction {
  type: "UPDATE_ASSISTANT_SOURCE";
  payload: {
    source: Source;
  };
}

type ConversationAction =
  | AddMessageAction
  | UpdateAssistantMessageAction
  | UpdateAssistantSourceAction;
const conversationReducer = (
  state: Conversation,
  action: ConversationAction
): Conversation => {
  switch (action.type) {
    case "ADD_MESSAGE":
      return [...state, action.payload];

    case "UPDATE_ASSISTANT_SOURCE":
      return state.map((message, index) => {
        if (index === state.length - 1 && message.Assistant) {
          return {
            ...message,
            Assistant: [
              message.Assistant[0],
              message.Assistant[1].concat([action.payload.source]),
            ],
          };
        }
        return message;
      });
    case "UPDATE_ASSISTANT_MESSAGE":
      return state.map((message, index) => {
        if (index === state.length - 1 && message.Assistant) {
          return {
            ...message,
            Assistant: [
              message.Assistant[0] + action.payload.content,
              message.Assistant[1],
            ],
          };
        }
        return message;
      });
    default:
      return state;
  }
};

function App() {
  const [inputText, setInputText] = useState<string>("");
  const [conversation, dispatch] = useReducer(conversationReducer, []);
  const messagesEndRef = useRef<null | HTMLDivElement>(null);
  const [tooltip, setTooltip] = useState<{
    visible: boolean;
    x: number;
    y: number;
    text: string;
  }>({ visible: false, x: 0, y: 0, text: "" });

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [conversation, tooltip.visible]);

  async function handleMouseEvents(
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
    const userMessage = { User: inputText };
    dispatch({ type: "ADD_MESSAGE", payload: userMessage });
    setInputText("");
    // Open a connection to the SSE endpoint
    const source = new SSE(
      "https://oracle-rs.semanticallyinvalid.net/streaming_conversation",
      {
        headers: { "Content-Type": "application/json" },
        payload: JSON.stringify([...conversation, userMessage]),
      }
    );
    let emptyAssistant: [string, []] = ["", []];
    const emptyResponse = { Assistant: emptyAssistant };
    dispatch({ type: "ADD_MESSAGE", payload: emptyResponse });
    // Handle the message events from the server
    source.onmessage = (event) => {
      const data: PartialAssistant = JSON.parse(event.data);

      if (data.content) {
        dispatch({
          type: "UPDATE_ASSISTANT_MESSAGE",
          payload: {
            content: data.content,
          },
        });
      }
      if (data.source) {
        dispatch({
          type: "UPDATE_ASSISTANT_SOURCE",
          payload: {
            source: data.source,
          },
        });
      }
      if (data.finished) {
        source.close();
      }
    };
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
                  <AssistantResponse text={message.Assistant[0]} />
                  <ul className="reference-list">
                    {message.Assistant[1].map((source, urlIdx) => (
                      <li
                        id={"citation_" + source.ordinal.toString()}
                        key={urlIdx}
                        style={{ listStyleType: "none" }}
                      >
                        <div
                          className="link-bubble"
                          onClick={(e) =>
                            handleMouseEvents(e, source.origin_text)
                          }
                        >
                          <p className="reference">
                            [{source.ordinal}]:{" "}
                            <a href={source.url} target="_blank">
                              {source.citation}
                            </a>
                          </p>
                        </div>
                      </li>
                    ))}
                  </ul>
                  {tooltip.visible && (
                    <div className="tooltip-text">
                      <p>{tooltip.text}</p>
                    </div>
                  )}
                </div>
              );
            }
            return null; // this is just to handle cases where neither user nor assistant properties are present
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
