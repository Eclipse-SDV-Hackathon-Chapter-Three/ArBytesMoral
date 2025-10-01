    // We listen for the Ambient Light Color Data Updates
    match common::ClientTraitV2::subscribe(
        &mut v2_client,
        vec!["Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide.Color".to_owned()],
        None,
        None,
    )
    .await
    {
        Ok(mut stream) => {
            println!("Successfully subscribed to {:?}!", "Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide");
            tokio::spawn(async move {
                match stream.message().await {
                    Ok(option) => {
                        let response = option.unwrap();
                        for entry_update in response.entries {
                            let datapoint = entry_update.1;
                            println!("Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide: {datapoint:?}");
                        }
                    }
                    Err(err) => {
                        println!("Error: Could not receive response {err:?}");
                    }
                }
            });
        }
        Err(err) => {
            println!("Failed to subscribe to {:?}: {:?}", "Vehicle.Cabin.Light.AmbientLight.Row1.DriverSide", err);
        }
    }