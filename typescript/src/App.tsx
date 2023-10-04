import React, { useEffect, useRef, useState } from 'react';
import './App.css';

interface Message {
  User?: string;
  Assistant?: [string, string[]];
}

interface Conversation extends Array<Message> {};

function App() {
  const [inputText, setInputText] = useState<string>('');
  const [conversation, setConversation] = useState<Conversation>([]);
  const messagesEndRef = useRef<null | HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [conversation]);

  async function submit() {
    const userMessageArray = [{ User: inputText }];

    // First we add the message from the user
    setConversation((prev) => [...prev, ...userMessageArray]);
    setInputText('');

    // Then we send the message to the server and handle the response
    try {
      const response = await fetch('http://0.0.0.0:5000/conversation', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Access-Control-Allow-Origin': '*',
          'Access-Control-Allow-Methods': 'POST'
        },
        body: JSON.stringify([...conversation, ...userMessageArray]) // <--- Changed here!
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const serverResponse: Conversation = await response.json();
      setConversation(serverResponse);
    } catch (error) {
      console.error('Fetching failed:', error);
    }
  }

  return (
    <div className="App">
      <header className="App-header">
        <div className="message-list">
          {conversation.map((message, idx) => {
            if (message.User) {
              return <div key={idx} className="user-text"><p>{message.User}</p></div>;
            }
            if (message.Assistant) {
              return (
                <div key={idx} className="assistant-text">
                  <p>{message.Assistant[0]}</p>
                  <ul style={{ display: 'flex', flexDirection: 'row', gap: '8px' }}>
                    {message.Assistant[1].map((url, urlIdx) => (
                      <li key={urlIdx} style={{ listStyleType: 'none' }}>
                        <div className="link-bubble">
                          <a href={url}>{url}</a>
                        </div>
                      </li>
                    ))}
                  </ul>
                </div>
              );
            }
            return null;  // this is just to handle cases where neither User nor Assistant properties are present, though it shouldn't occur based on your data structure
          })}
          <div ref={messagesEndRef} />
        </div>

        <div style={{ position: 'fixed', bottom: 0, display: 'flex', width: '100%', padding: 16, boxSizing: 'border-box', backgroundColor: '#282c34' }}>
          <input type="text" onChange={(e) => setInputText(e.target.value)} value={inputText} style={{ width: '80%', marginRight: 8, padding: 8, borderRadius: 4, border: 'none' }} />
          <button onClick={submit} style={{ width: '20%', backgroundColor: '#61dafb', border: 'none', color: '#282c34', padding: 8, borderRadius: 4 }}>Submit</button>
        </div>
      </header>
    </div>
  );
}

export default App;