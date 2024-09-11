let render param =
  <html>
  <head>
      <script src="https://unpkg.com/htmx.org@2.0.2"
              integrity="sha384-Y7hw+L/jvKeWIRRkqWYfPcvVxHzVzn5REgzbawhxAuQGwX1XWe70vji+VSeHOThJ"
              crossorigin="anonymous">
      </script>
  </head>
  <body>
    <h1>The URL parameter was <%s param %>!</h1>
  </body>
  </html>
