#define_import_path template::distance

#import template::constants::{
    WGS84_A, WGS84_B, WGS84_F, EARTH_RADIUS, DEG_TO_RAD
}

// Haversine formula for spherical earth
fn haversine_distance(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
    let phi1 = lat1 * DEG_TO_RAD;
    let phi2 = lat2 * DEG_TO_RAD;
    let dphi = (lat2 - lat1) * DEG_TO_RAD;
    let dlambda = (lon2 - lon1) * DEG_TO_RAD;

    let a = sin(dphi * 0.5) * sin(dphi * 0.5) +
            cos(phi1) * cos(phi2) *
            sin(dlambda * 0.5) * sin(dlambda * 0.5);
    let c = 2.0 * atan2(sqrt(a), sqrt(1.0 - a));

    return EARTH_RADIUS * c;
}

// Vincenty's inverse formula for ellipsoidal earth
fn vincenty_distance(lat1: f32, lon1: f32, lat2: f32, lon2: f32) -> f32 {
    let phi1 = lat1 * DEG_TO_RAD;
    let phi2 = lat2 * DEG_TO_RAD;
    let L = (lon2 - lon1) * DEG_TO_RAD;

    let U1 = atan((1.0 - WGS84_F) * tan(phi1));
    let U2 = atan((1.0 - WGS84_F) * tan(phi2));
    let sinU1 = sin(U1);
    let cosU1 = cos(U1);
    let sinU2 = sin(U2);
    let cosU2 = cos(U2);

    var lambda = L;
    var lambda_prev: f32;
    var iter = 0u;

    var sinLambda: f32;
    var cosLambda: f32;
    var sinSigma: f32;
    var cosSigma: f32;
    var sigma: f32;
    var sinAlpha: f32;
    var cos2Alpha: f32;
    var cos2SigmaM: f32;
    var C: f32;

    loop {
        if (iter >= 100u) { break; }

        sinLambda = sin(lambda);
        cosLambda = cos(lambda);
        sinSigma = sqrt((cosU2 * sinLambda) * (cosU2 * sinLambda) +
                        (cosU1 * sinU2 - sinU1 * cosU2 * cosLambda) *
                        (cosU1 * sinU2 - sinU1 * cosU2 * cosLambda));

        if (sinSigma == 0.0) { return 0.0; }

        cosSigma = sinU1 * sinU2 + cosU1 * cosU2 * cosLambda;
        sigma = atan2(sinSigma, cosSigma);
        sinAlpha = cosU1 * cosU2 * sinLambda / sinSigma;
        cos2Alpha = 1.0 - sinAlpha * sinAlpha;

        if (cos2Alpha < 1e-10) {
            cos2SigmaM = 0.0;
        } else {
            cos2SigmaM = cosSigma - 2.0 * sinU1 * sinU2 / cos2Alpha;
        }

        // isNan
        if (cos2SigmaM != cos2SigmaM) { cos2SigmaM = 0.0; }

        C = WGS84_F / 16.0 * cos2Alpha * (4.0 + WGS84_F * (4.0 - 3.0 * cos2Alpha));
        lambda_prev = lambda;
        lambda = L + (1.0 - C) * WGS84_F * sinAlpha *
                 (sigma + C * sinSigma * (cos2SigmaM + C * cosSigma *
                  (-1.0 + 2.0 * cos2SigmaM * cos2SigmaM)));

        if (abs(lambda - lambda_prev) < 1e-6) { break; }
        iter += 1u;
    }

    let u2 = cos2Alpha * (WGS84_A * WGS84_A - WGS84_B * WGS84_B) / (WGS84_B * WGS84_B);
    let A = 1.0 + u2 / 16384.0 * (4096.0 + u2 * (-768.0 + u2 * (320.0 - 175.0 * u2)));
    let B = u2 / 1024.0 * (256.0 + u2 * (-128.0 + u2 * (74.0 - 47.0 * u2)));
    let deltaSigma = B * sinSigma * (cos2SigmaM + B / 4.0 *
                     (cosSigma * (-1.0 + 2.0 * cos2SigmaM * cos2SigmaM) -
                      B / 6.0 * cos2SigmaM * (-3.0 + 4.0 * sinSigma * sinSigma) *
                      (-3.0 + 4.0 * cos2SigmaM * cos2SigmaM)));

    return WGS84_B * A * (sigma - deltaSigma);
}
