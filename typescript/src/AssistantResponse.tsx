import React, { useEffect, useState } from "react";
import { marked } from "marked";
import DOMPurify from "dompurify";

import "./App.css";

function AssistantResponse({ text }: { text: string }) {
  const [markup, setMarkup] = useState("");

  useEffect(() => {
    // Parse the markdown text
    const result = marked.parse(text);

    // Check if the result is a promise
    if (result instanceof Promise) {
      // If it's a promise, wait for it to resolve
      result
        .then((parsedText) => {
          const sanitizedMarkup = DOMPurify.sanitize(parsedText);
          setMarkup(sanitizedMarkup);
        })
        .catch((error) => {
          console.error("Error parsing markdown:", error);
        });
    } else {
      // If it's not a promise, sanitize and set the markup directly
      const sanitizedMarkup = DOMPurify.sanitize(result);
      setMarkup(sanitizedMarkup);
    }
  }, [text]);

  return <div dangerouslySetInnerHTML={{ __html: markup }} />;
}

export default AssistantResponse;
