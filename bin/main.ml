let () =
  Dream.run
    @@ Dream.logger
    @@ Dream.router [
        Dream.get "/" (fun _ -> Dream.html "Good morning, world!");
        Dream.get "/hello/:name" (fun req -> Dream.param req "name" |> Foo.render |> Dream.html);
    ]
